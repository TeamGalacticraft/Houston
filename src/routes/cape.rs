use actix_web::{delete, get, HttpRequest, HttpResponse, patch, post, web};
use actix_web::web::{scope, ServiceConfig};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::database::models::cape_model::CapeModel;
use crate::models::cape::{Cape, Category};
use crate::routes::ApiError;
use crate::utils::auth::check_is_admin_from_headers;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("capes")
            .service(get_cape)
            .service(create_cape)
            .service(update_cape)
    );
}

#[get("/{id}")]
pub async fn get_cape(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    let _id = info.into_inner().0.parse::<i64>().ok();

    if let Some(id) = _id {
        let _cape = CapeModel::get(id, &**pool).await?;

        if let Some(cape) = _cape {
            Ok(HttpResponse::Ok().json(Cape::from(cape)))
        } else {
            Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
                err: "not_found",
                msg: "cape not found"
            }))
        }
    } else {
        Ok(HttpResponse::BadRequest().json(crate::models::error::ApiError {
            err: "bad_request",
            msg: "invalid cape id"
        }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewCape {
    pub name: String,
    pub category: Category,
    pub texture_url: String,
    pub legacy_name: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct EditedCape {
    pub name: Option<String>,
    pub category: Option<Category>,
    pub texture_url: Option<String>,
    pub legacy_name: Option<Option<String>>
}

#[post("/")]
pub async fn create_cape(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<NewCape>
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;
    let mut trans = pool.begin().await?;

    CapeModel {
        id: 0,
        name: data.name.clone(),
        category: data.category.clone(),
        texture_url: data.texture_url.clone(),
        legacy_name: data.legacy_name.clone()
    }
        .insert(&mut trans)
        .await?;

    Ok(HttpResponse::Ok().json(()))
}

#[patch("/{id}")]
pub async fn update_cape(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    data: web::Json<EditedCape>,
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let _id = info.into_inner().0.parse::<i64>().ok();

    if let Some(id) = _id {
        let _cape = CapeModel::get(id, &**pool).await?;
        if let Some(cape) = _cape {
            if let Some(name) = &data.name {
                sqlx::query!(
                    "
                    update capes
                    set name = $1
                    where id = $2
                    ",
                    name,
                    id
                )
                    .execute(&**pool)
                    .await?;
            }

            if let Some(category) = &data.category {
                sqlx::query!(
                    "
                    update capes
                    set category = $1
                    where id = $2
                    ",
                    category.to_string(),
                    id
                )
                    .execute(&**pool)
                    .await?;
            }

            if let Some(texture) = &data.texture_url {
                sqlx::query!(
                    "
                    update capes
                    set texture_url = $1
                    where id = $2
                    ",
                    texture,
                    id
                )
                    .execute(&**pool)
                    .await?;
            }

            if let Some(legacy_name) = &data.legacy_name {
                sqlx::query!(
                    "
                    update capes
                    set legacy_name = $1
                    where id = $2
                    ",
                    legacy_name.to_owned(),
                    id
                )
                    .execute(&**pool)
                    .await?;
            }

            Ok(HttpResponse::Ok().json(Cape::from(cape)))
        } else {
            Ok(HttpResponse::NotFound().json(crate::models::error::ApiError {
                err: "not_found",
                msg: "cape not found"
            }))
        }

    } else {
        Ok(HttpResponse::BadRequest().json(crate::models::error::ApiError {
            err: "bad_request",
            msg: "invalid cape id"
        }))
    }
}

#[delete("/{id}")]
pub async fn delete_cape(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, ApiError> {
    check_is_admin_from_headers(req.headers(), &**pool).await?;

    let _id = info.into_inner().0.parse::<i64>().ok();

    if let Some(id) = _id {
        let mut trans = pool.begin().await?;
        CapeModel::remove(id, &mut trans).await?;
        trans.commit().await?;
        
        Ok(HttpResponse::Ok().json(()))
    } else {
        Ok(HttpResponse::BadRequest().json(crate::models::error::ApiError {
            err: "bad_request",
            msg: "invalid cape id"
        }))
    }
}
