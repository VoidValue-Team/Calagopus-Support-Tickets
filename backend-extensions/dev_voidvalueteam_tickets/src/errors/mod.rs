use axum::http::StatusCode;
use shared::response::{ApiResponse, ApiResponseResult};

pub fn api_error(error: anyhow::Error) -> ApiResponseResult {
    let message = error.to_string();
    let status = match message.as_str() {
        "ticket not found" | "department unavailable" => StatusCode::NOT_FOUND,
        "ticket closed" | "invalid status transition" => StatusCode::CONFLICT,
        "server is not accessible to this user" => StatusCode::FORBIDDEN,
        "attachments are disabled"
        | "too many attachments"
        | "attachment filename is missing"
        | "attachment filename is invalid"
        | "attachment type is not allowed"
        | "attachment is too large"
        | "no attachments provided"
        | "message uuid is missing"
        | "message not found"
        | "staff member is not eligible" => StatusCode::BAD_REQUEST,
        _ => {
            tracing::error!(error = ?error, "support ticket operation failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    ApiResponse::error(if status == StatusCode::INTERNAL_SERVER_ERROR {
        "support operation failed"
    } else {
        &message
    })
    .with_status(status)
    .ok()
}
