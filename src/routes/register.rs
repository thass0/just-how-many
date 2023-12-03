use actix_web::{web, Responder};
use sqlx::PgPool;
use serde::Deserialize;
use url::Url;
use uuid::Uuid;
use anyhow::Context;
use crate::utils::e500;

// Register a new page with a given URL.
// Returns the UUID that will be used to refer to that page.
#[tracing::instrument(
    name = "Register page by URL",
    skip(db_pool)
)]
pub async fn register(
    form: web::Form<RegisterPageForm>,
    db_pool: web::Data<PgPool>,
) -> actix_web::Result<impl Responder> {
    let url = form.into_inner().url;
    let page_id = insert_page(url, &db_pool).await.map_err(e500)?;
    Ok(web::Json(page_id))
}

#[derive(Debug, Deserialize)]
pub struct RegisterPageForm {
    url: Url,
}

#[tracing::instrument(
    name = "Insert a new page",
    skip(db_pool)
)]
async fn insert_page(
    url: Url,
    db_pool: &PgPool,
) -> anyhow::Result<Uuid> {
    let page_id = Uuid::new_v4();
    sqlx::query!(
	r#"
INSERT INTO pages (page_id, owner, url)
VALUES ($1, $2, $3)"#,
	page_id,
	Uuid::new_v4(),
	url.as_str(),
    )
	.execute(db_pool)
	.await
	.context("Failed to insert new page")?;
    Ok(page_id)
}
