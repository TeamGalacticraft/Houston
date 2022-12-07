use actix_web::{HttpResponse, Responder};
use crate::models::error::ApiError;

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(ApiError {
        err: "not_found",
        msg: "this route does not exist"
    })
}