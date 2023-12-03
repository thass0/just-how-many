use actix_web::{Responder, web};
use sqlx::PgPool;
use anyhow::Context;
use serde::Deserialize;
use url::Url;
use crate::utils::e500;

#[tracing::instrument(
    name = "Retrieve the hits a page has",
    skip(pg_pool)
)]
pub async fn hits(
    query: web::Query<HitsParams>,
    pg_pool: web::Data<PgPool>,
) -> actix_web::Result<impl Responder> {
    let hits = hits_of_page_url(query.into_inner().url, &pg_pool)
	.await
	.map_err(e500)?;
    Ok(web::Json(hits))
}

#[derive(Debug, Deserialize)]
pub struct HitsParams {
    url: Url,
}

#[tracing::instrument(
    name = "Get hits of page url",
    skip(pg_pool)
)]
async fn hits_of_page_url(
    url: Url,
    pg_pool: &PgPool,
) -> anyhow::Result<i32> {
    let record = sqlx::query!(
	r#"
SELECT hits
FROM pages
WHERE url = $1
"#,
	url.as_str(),
    )
	.fetch_one(pg_pool)
	.await
	.with_context(|| format!("Failed to get hits of page url: {}", url))?;
    Ok(record.hits)
}
