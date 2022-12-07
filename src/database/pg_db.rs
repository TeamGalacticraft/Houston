use std::time::Duration;
use log::info;
use sqlx::{Connection, PgConnection, PgPool, Postgres};
use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::PgPoolOptions;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    info!("Initializing db connection");
    let db_url = dotenvy::var("DB_URL").expect("`DB_URL` not in .env");

    let pool = PgPoolOptions::new()
        .min_connections(
            dotenvy::var("DB_MIN_CONN")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(0)
        )
        .max_connections(
            dotenvy::var("DB_MAX_CONN")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(0)
        )
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .connect(&db_url)
        .await?;

    Ok(pool)
}

pub async fn check_migrations() -> Result<(), sqlx::Error> {
    let db_url = dotenvy::var("DB_URL").expect("`DB_URL` not in .env");
    let db_url = db_url.as_str();

    if !Postgres::database_exists(db_url).await? {
        info!("Creating db");
        Postgres::create_database(db_url).await?;
    }

    let mut conn: PgConnection = PgConnection::connect(db_url).await?;
    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("Error while migrating database.");

    Ok(())
}