use axum::http::StatusCode;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::api_response::ApiResponse;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("authorization failed")]
    UnAuthorization,
    #[error("header to str: {0}")]
    HeaderToStr(#[from] header::ToStrError),
    #[error("ai: {0}")]
    Ai(#[from] async_openai::error::OpenAIError),
    #[error("no response from ai")]
    NoResponse,
    #[error("event not support")]
    NotSupport,
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("gitea: {0}")]
    Gitea(#[from] gitea_sdk::error::TeatimeError),
    #[error("gitea: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        log::error!("{self}");

        let (code, message) = match self {
            ApiError::UnAuthorization => (20001, "invalid request"),
            ApiError::HeaderToStr(_) => (20002, "header to str error"),
            ApiError::Ai(_) => (20003, "ai error"),
            ApiError::NoResponse => (20003, "ai error"),
            ApiError::NotSupport => (20004, "event not support"),
            ApiError::SerdeJson(_) => (20005, "invalid request"),
            ApiError::Gitea(_) => (20006, "gitea error"),
            ApiError::Reqwest(_) => (20006, "gitea error"),
        };
        let response = ApiResponse::new(code, message);

        (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
