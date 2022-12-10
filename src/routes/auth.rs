use actix_web::{get, HttpResponse};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Query, scope, ServiceConfig};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use crate::database::models::user_model::UserModel;
use crate::models::error::ApiError;
use crate::utils::auth::{get_profile_from_token, get_token_from_msa_code};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(scope("auth")
        .service(auth_begin)
        .service(auth_msa)
    );
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Env error")]
    Env(#[from] dotenvy::Error),
    #[error("An unknown DB error occurred: {0}")]
    SqlxDB(#[from] sqlx::Error),
    #[error("DB error: {0}")]
    DB(#[from] crate::database::models::DBError),
    #[error("JSON error: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Auth error: {0}")]
    Auth(#[from] crate::utils::auth::AuthError),
    #[error("Error communicating with Microsoft: {0}")]
    Microsoft(#[from] reqwest::Error),
    #[error("Uuid parse error: {0}")]
    Uuid(#[from] uuid::Error)
}

impl actix_web::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::SqlxDB(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::DB(..) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::SerDe(..) => StatusCode::BAD_REQUEST,
            AuthError::Auth(..) => StatusCode::UNAUTHORIZED,
            AuthError::Microsoft(..) => StatusCode::FAILED_DEPENDENCY,
            AuthError::Uuid(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(ApiError {
            err: match self {
                AuthError::Env(..) => "env_error",
                AuthError::SqlxDB(..) => "db_error",
                AuthError::DB(..) => "db_error",
                AuthError::SerDe(..) => "json_error",
                AuthError::Auth(..) => "auth_error",
                AuthError::Microsoft(..) => "microsoft_error",
                AuthError::Uuid(..) => "uuid_parse_error",
            },
            msg: &self.to_string(),
        })
    }
}

// http://localhost:8090/v1/auth/begin
#[get("/begin")]
pub async fn auth_begin() -> Result<HttpResponse, AuthError> {
    let client_id = dotenvy::var("MICROSOFT_CLIENT_ID")?;
    let url = format!(
        "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id={0}&response_type=code&scope={1}&redirect_uri=http%3A%2F%2Flocalhost%3A8090%2Fv1%2Fauth%2Fmicrosoft",
        client_id,
        "XboxLive.signin%20offline_access"
    );

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", &*url))
        .json(())
    )
}

#[derive(Serialize, Deserialize)]
pub struct MSAAuthCode {
    pub code: String,
}

// http://localhost:8090/v1/auth/microsoft?code=<code>
#[get("/microsoft")]
pub async fn auth_msa(
    Query(info): Query<MSAAuthCode>,
    pool: Data<PgPool>
) -> Result<HttpResponse, AuthError> {
    let mut trans = pool.begin().await?;
    let token = get_token_from_msa_code(info.code).await?;
    let profile = get_profile_from_token(&token).await?;

    let user_result = UserModel::get(Uuid::parse_str(profile.id.as_str())?, &mut *trans).await?;
    match user_result {
        Some(user) => {
            if user.username != profile.name {
                sqlx::query!(
                    "
                    update users
                    set username = $1
                    where id = $2
                    ",
                    profile.name,
                    user.id
                )
                    .execute(&**pool)
                    .await?;
            }
        },
        None => {
            UserModel {
                id: Uuid::parse_str(&profile.id)?,
                username: profile.name,
                avatar_url: format!("https://visage.surgeplay.com/face/512/{}", profile.id),
                roles: vec![],
                created: Utc::now(),
            }
                .insert(&mut trans)
                .await?;
            trans.commit().await?;
        }
    }
    let site_url = dotenvy::var("SITE_URL").unwrap();

    Ok(HttpResponse::TemporaryRedirect().append_header(("Location", format!("{}?code={}", &site_url, &token))).json(()))
}
