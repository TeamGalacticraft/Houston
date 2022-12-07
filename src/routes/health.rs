use actix_web::{get, HttpResponse};
use actix_web::web::Data;
use serde_json::json;
use sqlx::PgPool;
use crate::health::status::test_db;


#[get("/health")]
pub async fn health_get(client: Data<PgPool>) -> HttpResponse {
    // check database
    let db = test_db(client).await;
    if db.is_err() {
        return HttpResponse::InternalServerError().json(json!({
            "ready": false,
            "message": "DB connection error"
        }))
    }

    // it do be ready pog
    HttpResponse::Ok().json(json!({
        "ready": true,
        "message": "Ok"
    }))
}