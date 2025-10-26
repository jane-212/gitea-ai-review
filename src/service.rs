use axum::extract::State;
use axum::http::{HeaderMap, header};
use serde::Deserialize;
use serde_json::Value;

use crate::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::error::{ApiError, Result};

pub async fn webhook<'a>(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<ApiResponse<'a>> {
    let Some(authorization) = headers.get(header::AUTHORIZATION) else {
        return Err(ApiError::UnAuthorization);
    };
    let authorization = authorization.to_str()?;
    if authorization != app_state.gitea_authorization {
        return Err(ApiError::UnAuthorization);
    }

    let Some(event) = headers.get("X-GitHub-Event") else {
        return Err(ApiError::NotSupport);
    };
    let event = event.to_str()?;
    let event = Event::parse(event);

    match event {
        Event::PullRequest => {
            tokio::spawn(async move {
                if let Err(err) = review(app_state, body).await {
                    log::error!("{err}");
                }
            });
        }
        Event::Other => return Err(ApiError::NotSupport),
    }

    let response = ApiResponse::new(0, "success");
    Ok(response)
}

async fn review(app_state: AppState, body: String) -> Result<()> {
    let request: Value = serde_json::from_str(&body)?;

    let Some(action) = request["action"].as_str() else {
        return Err(ApiError::NotSupport);
    };
    let action = Action::parse(action);
    if matches!(action, Action::Other) {
        return Err(ApiError::NotSupport);
    }

    let Some(owner) = request["repository"]["owner"]["username"].as_str() else {
        return Err(ApiError::NotSupport);
    };
    let Some(repo) = request["repository"]["name"].as_str() else {
        return Err(ApiError::NotSupport);
    };
    let Some(index) = request["pull_request"]["number"].as_i64() else {
        return Err(ApiError::NotSupport);
    };

    let this_username = get_this_username(&app_state).await?;
    let (commit_id, state) =
        request_reviewer(&app_state, &this_username, owner, repo, index).await?;

    let diff = get_diff(&app_state, owner, repo, index).await?;
    let review = ai_review(&app_state, &diff).await?;

    send_review(&app_state, owner, repo, index, &review, &commit_id, &state).await?;

    Ok(())
}

async fn send_review(
    app_state: &AppState,
    owner: &str,
    repo: &str,
    index: i64,
    review: &str,
    commit_id: &str,
    state: &str,
) -> Result<()> {
    #[derive(Deserialize)]
    struct Review {
        findings: Vec<Finding>,
        overall_explanation: String,
    }

    #[derive(Deserialize)]
    struct Finding {
        body: String,
        code_location: CodeLocation,
    }

    #[derive(Deserialize)]
    struct CodeLocation {
        absolute_file_path: String,
        line: u32,
    }

    let line_count = review.lines().count();
    let review = review
        .lines()
        .skip(1)
        .take(line_count - 2)
        .collect::<String>();

    let review: Review = serde_json::from_str(&review)?;
    let comments = review
        .findings
        .into_iter()
        .map(|finding| {
            serde_json::json!({
                "body": finding.body,
                "new_position": finding.code_location.line,
                "old_position": 0,
                "path": finding.code_location.absolute_file_path
            })
        })
        .collect::<Vec<_>>();
    let event = match state {
        "APPROVED" => "COMMENT",
        "PENDING" => "APPROVED",
        "COMMENT" => "COMMENT",
        "REQUEST_CHANGES" => "APPROVED",
        "REQUEST_REVIEW" => "APPROVED",
        _ => "APPROVED",
    };

    let url = format!("repos/{owner}/{repo}/pulls/{index}/reviews");
    let json = serde_json::json!({
        "body": review.overall_explanation,
        "comments": comments,
        "commit_id": commit_id,
        "event": event
    });
    app_state.gitea_client.post(url).json(&json).send().await?;

    Ok(())
}

async fn request_reviewer(
    app_state: &AppState,
    username: &str,
    owner: &str,
    repo: &str,
    index: i64,
) -> Result<(String, String)> {
    let url = format!("repos/{owner}/{repo}/pulls/{index}/reviews");
    let reviews = app_state
        .gitea_client
        .get(url)
        .send()
        .await?
        .json::<Value>()
        .await?;
    let Some(reviews) = reviews.as_array() else {
        let error = format!("reviews not found");
        return Err(ApiError::Custom(error));
    };
    for review in reviews {
        let Some(user) = review["user"].as_object() else {
            continue;
        };
        let Some(review_username) = user["login"].as_str() else {
            continue;
        };
        let Some(commit_id) = review["commit_id"].as_str() else {
            continue;
        };
        let Some(state) = review["state"].as_str() else {
            continue;
        };

        if review_username == username {
            return Ok((commit_id.to_string(), state.to_string()));
        }
    }

    let url = format!("repos/{owner}/{repo}/pulls/{index}/requested_reviewers");
    let json = serde_json::json!({
        "reviewers": [
            username
        ],
        "team_reviewers": []
    });
    let review = app_state
        .gitea_client
        .post(url)
        .json(&json)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let Some(commit_id) = review["commit_id"].as_str() else {
        let error = format!("commit_id not found: {review}");
        return Err(ApiError::Custom(error));
    };
    let Some(state) = review["state"].as_str() else {
        let error = format!("state not found: {review}");
        return Err(ApiError::Custom(error));
    };

    Ok((commit_id.to_string(), state.to_string()))
}

async fn get_this_username(app_state: &AppState) -> Result<String> {
    let url = format!("user");
    let user = app_state
        .gitea_client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let Some(username) = user["login"].as_str() else {
        let error = format!("login not found: {user}");
        return Err(ApiError::Custom(error));
    };

    Ok(username.to_string())
}

async fn get_diff(app_state: &AppState, owner: &str, repo: &str, index: i64) -> Result<String> {
    let url = format!("repos/{owner}/{repo}/pulls/{index}.diff");
    let diff = app_state
        .gitea_client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(diff)
}

async fn ai_review(app_state: &AppState, diff: &str) -> Result<String> {
    let review_prompt = include_str!("../review.md");
    let message = format!("{review_prompt}\n\n{diff}");
    let review = app_state.ai_client.chat(message).await?;

    Ok(review)
}

enum Action {
    Opened,
    Synchronized,
    Other,
}

impl Action {
    fn parse(action: &str) -> Self {
        match action {
            "opened" => Self::Opened,
            "synchronized" => Self::Synchronized,
            _ => Self::Other,
        }
    }
}

enum Event {
    PullRequest,
    Other,
}

impl Event {
    fn parse(event: &str) -> Self {
        match event {
            "pull_request" => Self::PullRequest,
            _ => Self::Other,
        }
    }
}
