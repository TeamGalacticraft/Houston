use actix_web::web;
use sqlx::PgPool;

pub async fn test_db(
    pg: web::Data<PgPool>
) -> Result<(), sqlx::Error> {
    let mut trans = pg.acquire().await?;
    sqlx::query(
        "
        SELECT 1
        ",
    )
        .execute(&mut trans)
        .await
        .map(|_| ())
}