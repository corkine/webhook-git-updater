use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid length")]
    InvalidSize(#[from] std::num::TryFromIntError),
    #[error("Db error {0}")]
    DbError(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Resource Not found")]
    NotFound,
    #[error("Not found file: {0}")]
    NotFoundFile(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Git ops error: {0}")]
    GitOpsError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let json = json!({
            "error": self.to_string(),
            "code": -1
        });
        match *self {
            ApiError::Unauthorized => HttpResponse::Unauthorized()
                .append_header(("WWW-Authenticate", "Basic realm=\"Secure Area\""))
                .json(json),
            ApiError::NotFound => HttpResponse::build(StatusCode::NOT_FOUND).json(json),
            _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(json),
        }
    }
}
