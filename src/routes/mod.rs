mod index;
mod health;
mod not_found;
mod auth;
mod user;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use actix_web::body::BoxBody;
pub use auth::config as auth_config;
pub use user::config as user_config;
pub use self::index::index_get;
pub use self::health::health_get;
pub use self::not_found::not_found;

pub fn v1_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("v1")
            .configure(auth_config)
            .configure(user_config)
    );
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("environment error")]
    Env(#[from] dotenvy::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("database error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("database error: {0}")]
    Database(#[from] crate::database::models::DBError),
    #[error("auth error: {0}")]
    Authentication(#[from] crate::utils::auth::AuthError),
    #[error("uuid parse error: {0}")]
    Uuid(#[from] uuid::Error)
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Json(..) => StatusCode::BAD_REQUEST,
            ApiError::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::Uuid(..) => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(
            crate::models::error::ApiError {
                err: match self {
                    ApiError::Env(..) => "env_error",
                    ApiError::Json(..) => "json_error",
                    ApiError::SqlxDatabase(..) => "database_error",
                    ApiError::Database(..) => "database_error",
                    ApiError::Authentication(..) => "authentication",
                    ApiError::Uuid(..) => "uuid_parse_error",
                },
                msg: &self.to_string(),
            }
        )
    }
}