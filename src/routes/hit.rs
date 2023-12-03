use actix_web::{HttpRequest, HttpResponse, web};
use sqlx::PgPool;
use anyhow::Context;
use uuid::Uuid;
use crate::utils::{e500, RedisPool, hash_data};
use redis::Commands;
use std::time::{SystemTime, UNIX_EPOCH};

#[tracing::instrument(
    name = "Register page hit",
    skip(pg_pool, redis_pool, req, visit_duration)
)]
pub async fn hit(
    req: HttpRequest,
    path: web::Path<Uuid>,
    visit_duration: web::Data<u64>,
    pg_pool: web::Data<PgPool>,
    redis_pool: web::Data<RedisPool>,
) -> actix_web::Result<HttpResponse> {
    let page_id: uuid::Uuid = path.into_inner();

    let addr = hash_data(&req.peer_addr()
			 .ok_or_else(|| e500("Missing IP address"))?
			 .ip());

    let visit = check_in_visitor(page_id, addr,
				 *visit_duration.get_ref(), &redis_pool)
	.await
        .map_err(e500)?;
    if visit == VisitStatus::New {
	increment_hit(page_id, &pg_pool)
	    .await
	    .map_err(e500)?;
    }

    
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Increment page hits",
    skip (pg_pool)
)]
pub async fn increment_hit(
    page_id: uuid::Uuid,
    pg_pool: &PgPool,
) -> anyhow::Result<()>
{
    // i64 because of sqlx' type constraints.
    let now: i64 = unix_time_secs()
	.try_into()
	.expect("It's not 2100");
    sqlx::query!(
	r#"
UPDATE pages
SET hits = hits + 1,
    timestamps = ARRAY_APPEND(timestamps, $1)
WHERE page_id = $2"#,
	now,
	page_id
    )
	.execute(pg_pool)
	.await
	.context("Failed to increase hits count")?;
    Ok(())
}

/// Check if the given IP address has been seen before
/// in the last 12 hours.
#[tracing::instrument(
    name = "Check-in visiting IP address",
    skip(redis_pool)
)]
async fn check_in_visitor(
    page_id: Uuid,
    addr: u64,
    visit_duration: u64,
    redis_pool: &RedisPool
) -> anyhow::Result<VisitStatus> {
    let page_id = &page_id.to_string();
    let addr = &addr.to_string();
    let mut con = redis_pool.get()
        .context("Failed to retrieve a connection")?;

    let now = unix_time_secs();

    if con.hexists(page_id, addr)? {
	let expiry: u64 = con.hget(page_id, addr)?;
	if now - expiry > visit_duration {
	    tracing::info!("New visit (but seen before)");
	    con.hset(page_id, addr, now.to_string())?;
	    Ok(VisitStatus::New)
	} else {
	    tracing::info!("Visitor has been seen before");
	    Ok(VisitStatus::Old)
	}
    } else {
	tracing::info!("New visit");
	con.hset(page_id, addr, now.to_string())?;
	Ok(VisitStatus::New)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum VisitStatus {
    /// Visitor hasn't been seen in the last twelve hours.
    New,
    /// Visitor has been seen in the last twelve hours.
    Old,
}

fn unix_time_secs() -> u64 {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
