use actix_web::{HttpRequest, HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;
use uuid::Uuid;
use crate::utils::e500;

#[tracing::instrument(
    name = "Register page hit",
    skip(db_pool)
)]
pub async fn hit(
    req: HttpRequest,
    path: web::Path<Uuid>,
    db_pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let page_id: uuid::Uuid = path.into_inner();

    let _addr = req.peer_addr()
	.ok_or_else(|| e500("Missing IP address"))?;

    increment_hit(page_id, &db_pool)
        .await
        .map_err(e500)?;
    
    Ok(HttpResponse::Ok().finish())
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
