use actix_web::{Responder, web};
use sqlx::PgPool;
use anyhow::Context;

#[tracing::instrument(
    name = "Register page hit",
    skip(db_pool)
)]
pub async fn hit(
    path: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let page_id: uuid::Uuid = path.into_inner().parse().unwrap();
    tracing::info!("Got {}", page_id);

    increment_hit(page_id, &db_pool)
        .await
        .expect("Database should not fail");
    
    format!("Welcome {page_id}")
}

#[tracing::instrument(
    name = "Increment page hits",
    skip (db_pool)
)]
pub async fn increment_hit(
    page_id: uuid::Uuid,
    db_pool: &PgPool,
) -> Result<(), anyhow::Error>
{
    sqlx::query!(
	r#"
UPDATE pages
SET hits = hits + 1
WHERE page_id = $1"#,
	page_id
    )
	.execute(db_pool)
	.await
	.context("Failed to increase hits count")?;
    Ok(())
}
