use std::net::TcpListener;
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use tracing_actix_web::TracingLogger;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::routes;
use crate::configuration::{Settings, PostgresSettings, RedisSettings};
use crate::utils::RedisPool;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let postgres = get_pg_connection_pool(&configuration.postgres).await;
	let redis = get_redis_connection_pool(&configuration.redis);
        let address = format!(
            "{}:{}",
            configuration.application.host,
            configuration.application.port,
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            postgres,
	    redis,
	    configuration.application.visit_duration,
        ).await?;

        Ok(Self{ port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}


pub async fn get_pg_connection_pool(
    configuration: &PostgresSettings,
) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn get_redis_connection_pool(
    configuration: &RedisSettings,
) -> r2d2::Pool<redis::Client> {
    let client = redis::Client::open(configuration.with_db()).unwrap();
    r2d2::Pool::builder()
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(client)
	.unwrap()
}


pub async fn run(
    listener: TcpListener,
    pg: PgPool,
    redis: RedisPool,
    visit_duration: u64,
) -> Result<Server, anyhow::Error> {
    let pg = web::Data::new(pg);
    let redis = web::Data::new(redis);
    let visit_duration = web::Data::new(visit_duration);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(routes::health_check))
            .route("/hit/{site_id}", web::get().to(routes::hit))
            .route("/register", web::post().to(routes::register))
            .route("/hits", web::get().to(routes::hits))
            .app_data(pg.clone())
            .app_data(redis.clone())
            .app_data(visit_duration.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
