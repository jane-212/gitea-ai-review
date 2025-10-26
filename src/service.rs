use axum::extract::State;
use axum::http::{HeaderMap, header};
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

    let url = format!("repos/{owner}/{repo}/pulls/{index}.diff");
    let diff = app_state
        .gitea_client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let review_prompt = include_str!("review.md");
    let message = format!("{review_prompt}\n\n{diff}");
    let review = app_state.ai_client.chat(message).await?;

    let url = format!("/repos/{owner}/{repo}/issues/{index}/comments");
    let json = serde_json::json!({
        "body": review,
        "comments": [
            {
                "body": "comment body1",
                "new_position": 1,
                "old_position": 0,
                "path": "main.py",
            },
            {
                "body": "comment body2",
                "new_position": 0,
                "old_position": 1,
                "path": "main.py",
            }
        ],
        "commit_id": review_id,
        "event": "PENDING"
    });
    app_state
        .gitea_client
        .post(url)
        .json(&json)
        .send()
        .await?
        .error_for_status()?;

    let url = format!("repos/{owner}/{repo}/pulls/{index}/reviews/{review_id}");
    let json = serde_json::json!({
        "body": "review body",
        "event": "COMMENT"        
    });
    app_state
        .gitea_client
        .post(url)
        .json(&json)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
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
