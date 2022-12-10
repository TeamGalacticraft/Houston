mod routes;
mod health;
mod models;
mod utils;
mod database;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use env_logger::Env;
use log::{error, info, warn};
use utils::env::parse_var;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    info!("Trans rights are human rights.");

    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    if validate_env() {
        error!("Failed to validate required env vars.");
    }

    let sentry = sentry::init(());
    if sentry.is_enabled() {
        info!("Sentry integration enabled.");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    info!(
        "Starting Houston on {}",
        dotenvy::var("BIND_ADDR").unwrap()
    );

    database::check_migrations()
        .await
        .expect("failed to run DB migrations.");

    let pool = database::connect()
        .await
        .expect("DB connection failed.");

    info!("Starting HTTP server.");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600)
            .send_wildcard();

        App::new()
            .wrap(cors)
            .wrap(middleware::NormalizePath::trim())
            .wrap(sentry_actix::Sentry::new())
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::v1_config)
            .service(routes::index_get)
            .service(routes::health_get)
            .default_service(web::get().to(routes::not_found))
    })
        .bind(dotenvy::var("BIND_ADDR").unwrap())?
        .run()
        .await
}

fn validate_env() -> bool {
    let mut failed = false;

    fn check<T: std::str::FromStr>(var: &'static str) -> bool {
        let missing = parse_var::<T>(var).is_none();
        if missing {
            warn!(
                "`{}` missing from .env or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
        }
        missing
    }

    failed |= check::<String>("BIND_ADDR");
    failed |= check::<String>("SITE_URL");
    failed |= check::<String>("DATABASE_URL");

    failed |= check::<String>("MICROSOFT_CLIENT_ID");
    failed |= check::<String>("MICROSOFT_CLIENT_SECRET");

    failed
}