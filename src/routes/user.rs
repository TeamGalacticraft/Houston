use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::web::{scope, ServiceConfig};
use sqlx::PgPool;
use uuid::Uuid;
use crate::database::models::user_model::UserModel;
use crate::routes::ApiError;
use crate::utils::auth::get_user_from_headers;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("users")
            .service(current_user)
            .service(get_user)
    );
}

#[get("/@me")]
pub async fn current_user(
    req: HttpRequest,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok()
        .json(get_user_from_headers(req.headers(), &**pool).await?))
}

#[get("/{uuid}")]
pub async fn get_user(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    let id = Uuid::parse_str(&*info.into_inner().0)?;
    let user_data = UserModel::get(id, &**pool).await?;

    if let Some(data) = user_data {
        let resp: crate::models::user::User = data.into();
        Ok(HttpResponse::Ok().json(resp))
    } else {
        Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
            err: "not_found",
            msg: "user not found",
        }))
    }
}
