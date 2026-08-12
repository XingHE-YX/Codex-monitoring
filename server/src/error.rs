use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    AuthInvalid,
    ValidationFailed,
    PairingCodeInvalid,
    PairingCodeExpired,
    PairingCodeUsed,
    PairingLocked,
    DeviceAlreadyBound,
    RateLimited,
    NotFound,
    Conflict,
    SourceUnavailable,
    ModelUnavailable,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
    pub request_id: String,
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
        }
    }

    pub fn response(&self, request_id: impl Into<String>) -> ErrorResponse {
        ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message.clone(),
                retryable: self.retryable,
            },
            request_id: request_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AppError, ErrorCode};

    #[test]
    fn public_error_serializes_with_required_envelope_fields() {
        let response = AppError::new(
            ErrorCode::AuthInvalid,
            "The device credential is invalid or revoked.",
            false,
        )
        .response("request-123");

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "error": {
                    "code": "AUTH_INVALID",
                    "message": "The device credential is invalid or revoked.",
                    "retryable": false
                },
                "request_id": "request-123"
            })
        );
    }
}
