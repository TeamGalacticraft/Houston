use actix_web::{get, HttpRequest, HttpResponse, patch, web};
use actix_web::web::{scope, ServiceConfig};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::database::models::user_model::UserModel;
use crate::models::user::Role;
use crate::routes::ApiError;
use crate::utils::auth::{check_is_admin_from_headers, get_user_from_headers};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("users")
            .service(current_user)
            .service(get_user)
            .service(update_user)
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

#[derive(Serialize, Deserialize)]
pub struct EditedUser {
    pub roles: Option<Vec<Role>>,
}

#[patch("/{uuid}")]
pub async fn update_user(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    data: web::Json<EditedUser>
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let id = Uuid::parse_str(&*info.into_inner().0)?;
    let user_data = UserModel::get(id, &**pool).await?;

    if let Some(mut model) = user_data {
        if let Some(roles) = &data.roles {
            model.roles = roles.to_owned();
            let role_strings: Vec<String>
                = model.roles.iter().map(|x| x.to_string()).collect::<Vec<String>>();
            sqlx::query!(
                "
                update users
                set roles = $1
                where id = $2
                ",
                &role_strings,
                id
            )
                .execute(&**pool)
                .await?;

            Ok(HttpResponse::Ok().json(data))
        } else {
            Ok(HttpResponse::BadRequest().json(crate::models::error::ApiError {
                err: "bad_request",
                msg: "nothing to update",
            }))
        }
    } else {
        Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
            err: "not_found",
            msg: "user not found",
        }))
    }
}