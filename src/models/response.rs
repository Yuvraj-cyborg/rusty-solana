use serde::Serialize;
use axum::{response::{IntoResponse, Response}, Json, http::StatusCode};

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Debug)]
pub enum ApiResponse<T> {
    Success(T),
    Error(String),
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success(data) => {
                (StatusCode::OK, Json(SuccessResponse {
                    success: true,
                    data,
                })).into_response()
            }
            ApiResponse::Error(error) => {
                (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    success: false,
                    error,
                })).into_response()
            }
        }
    }
}

pub fn json_success<T: Serialize>(data: T) -> Json<SuccessResponse<T>> {
    Json(SuccessResponse { success: true, data })
}

pub fn json_error(msg: &str) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        success: false,
        error: msg.to_string(),
    })
}
