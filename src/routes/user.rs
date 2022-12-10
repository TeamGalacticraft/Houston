use std::collections::HashMap;
use actix_web::{delete, get, HttpRequest, HttpResponse, patch, route, web};
use actix_web::web::{scope, ServiceConfig};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::database::models::player_cape_model::{LegacyPlayerCapeModel, PlayerCapeModel};
use crate::database::models::user_model::UserModel;
use crate::models::cape::Cape;
use crate::models::user::Role;
use crate::routes::ApiError;
use crate::utils::auth::{check_is_admin_from_headers, check_is_self_or_admin_from_headers, get_user_from_headers};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("users")
            .service(current_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_cape)
            .service(list_capes)
            .service(list_capes_legacy)
    );
}

#[get("/@me")]
pub async fn current_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok()
        .json(get_user_from_headers(req.headers(), &**pool).await?))
}

#[get("/{uuid}")]
pub async fn get_user(
    info: web::Path<(String, )>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = Uuid::parse_str(&info.into_inner().0)?;
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
    info: web::Path<(String, )>,
    pool: web::Data<PgPool>,
    data: web::Json<EditedUser>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let id = Uuid::parse_str(&info.into_inner().0)?;
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

#[delete("/{uuid}")]
pub async fn delete_user(
    req: HttpRequest,
    info: web::Path<(String, )>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = Uuid::parse_str(&info.into_inner().0)?;
    let _user = UserModel::get(id, &**pool).await?;

    if let Some(_) = _user {
        check_is_self_or_admin_from_headers(req.headers(), id, &**pool).await?;

        let mut trans = pool.begin().await?;
        UserModel::remove(id, &mut trans).await?;
        trans.commit().await?;

        Ok(HttpResponse::Ok().json(()))
    } else {
        Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
            err: "not_found",
            msg: "user not found",
        }))
    }
}

#[get("/{uuid}/cape")]
pub async fn get_cape(
    info: web::Path<(Uuid, )>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    let _cape = PlayerCapeModel::get_cape_for(id, &**pool).await?;

    if let Some(cape) = _cape {
        Ok(HttpResponse::Ok().json(Cape::from(cape)))
    } else {
        Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
            err: "not_found",
            msg: "player does not have a cape set",
        }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct EditedCape {
    pub id: i64,
}

#[route("/{uuid}/cape", method = "POST", method = "PATCH")]
pub async fn update_cape(
    req: HttpRequest,
    info: web::Path<(Uuid, )>,
    pool: web::Data<PgPool>,
    data: web::Json<EditedCape>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    check_is_self_or_admin_from_headers(req.headers(), id, &**pool).await?;
    let _player_cape = PlayerCapeModel::get_cape_for(id, &**pool).await?;

    if let Some(_) = _player_cape {
        sqlx::query!(
            "
            update player_capes
            set cape = $1
            where player = $2
            ",
            data.id,
            id
        )
            .execute(&**pool)
            .await?;
    } else {
        let mut trans = pool.begin().await?;
        PlayerCapeModel {
            player: id,
            cape: Some(data.id),
        }
            .insert(&mut trans)
            .await?;
    }

    let cape = PlayerCapeModel::get_cape_for(id, &**pool).await?
        .expect("cape missing after insert");

    Ok(HttpResponse::Ok().json(Cape::from(cape)))
}

#[delete("/{uuid}/cape")]
pub async fn delete_cape(
    req: HttpRequest,
    info: web::Path<(Uuid, )>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let id = info.into_inner().0;
    check_is_self_or_admin_from_headers(req.headers(), id, &**pool).await?;

    let mut trans = pool.begin().await?;
    PlayerCapeModel::remove(id, &mut trans).await?;
    trans.commit().await?;

    Ok(HttpResponse::Ok().json(()))
}

pub fn format_legacy_text(
    capes: Vec<LegacyPlayerCapeModel>
) -> String {
    let mut out = "".to_owned();

    for cape in capes {
        if out != "" {
            out.push_str("\n");
        }
        out.push_str(cape.uuid.to_string().replace("-", "").as_str());
        out.push_str(":");
        out.push_str(cape.cape.as_str());
        out.push_str(" ");
        out.push_str(cape.name.as_str());
    }

    out.to_string()
}

#[get("/capes")]
pub async fn list_capes(
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    let capes = PlayerCapeModel::get_list(&**pool).await?;
    Ok(HttpResponse::Ok().json(capes.iter().map(|x| {
        (*x.0, Cape::from(*x.1))
    }).collect::<HashMap<Uuid, Cape>>()))
}

#[get("/capes/legacy")]
pub async fn list_capes_legacy(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let capes = PlayerCapeModel::get_legacy_list(&**pool).await?;

    if req.headers().contains_key("Accepts") {
        let accepts_type = req.headers().get("Accepts").expect("");
        if accepts_type.eq("application/json") {
            Ok(HttpResponse::Ok().json(capes))
        } else {
            Ok(HttpResponse::Ok().body(format_legacy_text(capes)))
        }
    } else {
        Ok(HttpResponse::Ok().body(format_legacy_text(capes)))
    }
}