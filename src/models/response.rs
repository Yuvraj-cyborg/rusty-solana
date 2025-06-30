use serde::Serialize;

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

pub fn json_success<T: Serialize>(data: T) -> axum::Json<SuccessResponse<T>> {
    axum::Json(SuccessResponse { success: true, data })
}

pub fn json_error(msg: &str) -> axum::Json<ErrorResponse> {
    axum::Json(ErrorResponse {
        success: false,
        error: msg.to_string(),
    })
}
