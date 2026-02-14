use actix_web::HttpResponse;
use serde::Serialize;
use std::fmt;

/// Unified application error type.
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Elasticsearch(String),
    RateLimitExceeded {
        tier: String,
        limit: u64,
        reset_at: String,
    },
    Unauthorized(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Elasticsearch(msg) => write!(f, "Elasticsearch error: {}", msg),
            Self::RateLimitExceeded { tier, limit, .. } => {
                write!(f, "Rate limit exceeded for {} tier ({}/hour)", tier, limit)
            }
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    success: bool,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: u16,
    message: String,
}

impl AppError {
    /// Convert to an HTTP response with proper status code and JSON body.
    pub fn to_response(&self) -> HttpResponse {
        let (status, code, message) = match self {
            Self::NotFound(msg) => (
                actix_web::http::StatusCode::NOT_FOUND,
                404,
                msg.clone(),
            ),
            Self::Elasticsearch(msg) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                500,
                format!("Service temporarily unavailable: {}", msg),
            ),
            Self::RateLimitExceeded { tier, limit, reset_at } => {
                let resp = HttpResponse::TooManyRequests()
                    .insert_header(("X-RateLimit-Limit", limit.to_string()))
                    .insert_header(("X-RateLimit-Remaining", "0"))
                    .insert_header(("X-RateLimit-Reset", reset_at.as_str()))
                    .json(ErrorBody {
                        success: false,
                        error: ErrorDetail {
                            code: 429,
                            message: format!(
                                "Rate limit exceeded. Your {} plan allows {} requests per hour. Resets at {}. Upgrade your plan for higher limits.",
                                tier, limit, reset_at
                            ),
                        },
                    });
                return resp;
            }
            Self::Unauthorized(msg) => (
                actix_web::http::StatusCode::FORBIDDEN,
                403,
                msg.clone(),
            ),
            Self::Internal(msg) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                500,
                msg.clone(),
            ),
        };

        HttpResponse::build(status).json(ErrorBody {
            success: false,
            error: ErrorDetail { code, message },
        })
    }
}
