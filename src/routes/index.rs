use actix_web::{get, HttpResponse};
use serde_json::json;

#[get("/")]
pub async fn index_get() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "name": "galacticraft-houston",
        "version": env!("CARGO_PKG_VERSION"),
        "documentation": "https://capes.galacticraft.team/docs",
        "about": "hey there astronaut 🚀"
    }))
}