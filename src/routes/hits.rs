use actix_web::{Responder, web};
use sqlx::PgPool;
use anyhow::Context;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Hits {
    pub n: i32,
    pub timestamps: Vec<i64>,
}

#[tracing::instrument(
    name = "Get hits of page url",
    skip(pg_pool)
)]
async fn hits_of_page_url(
    url: Url,
    pg_pool: &PgPool,
) -> anyhow::Result<Hits> {
    let record = sqlx::query!(
	r#"
SELECT hits, timestamps
FROM pages
WHERE url = $1
"#,
	url.as_str(),
    )
	.fetch_one(pg_pool)
	.await
	.with_context(|| format!("Failed to get hits of page url: {}", url))?;
    Ok(Hits {
	n: record.hits,
	timestamps: record.timestamps.unwrap_or_else(|| vec![]),
    })
}
