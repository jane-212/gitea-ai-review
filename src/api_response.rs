use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<'a> {
    code: u16,
    message: &'a str,
}

impl<'a> ApiResponse<'a> {
    pub fn new(code: u16, message: &'a str) -> Self {
        Self { code, message }
    }
}

impl<'a> IntoResponse for ApiResponse<'a> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
