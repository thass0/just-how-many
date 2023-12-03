use uuid::Uuid;
use once_cell::sync::Lazy;
use sqlx::{PgPool, PgConnection, Connection, Executor};

use jhm::startup::{Application, get_pg_connection_pool};
use jhm::configuration::{PostgresSettings, get_configuration};
use jhm::telemetry::*;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_name = "test".to_owned();
    let default_level = "info".to_owned();

    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(
            default_name,
            default_level,
            std::io::stdout,
        ));
    } else {
        init_subscriber(get_subscriber(
            default_name,
            default_level,
            std::io::sink
        ));
    }
});


pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db: PgPool,
    api_client: reqwest::Client,
}

impl TestApp {
    pub async fn spawn() -> Self {
        Lazy::force(&TRACING);

        let configuration = {
            let mut c = get_configuration().expect("Failed to read configuratoin");
            c.postgres.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
	    c.application.visit_duration = 1;
            c
        };

        Self::configure_postgres(&configuration.postgres).await;

        let application = Application::build(configuration.clone())
            .await
            .expect("Failed to build application");
        let application_port = application.port();
        let _ = tokio::spawn(application.run_until_stopped());

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        Self {
            address: format!("http://127.0.0.1:{}", application_port),
            port: application_port,
            db: get_pg_connection_pool(&configuration.postgres).await,
            api_client: client,
        }
    }

    async fn configure_postgres(db_config: &PostgresSettings) -> PgPool {
        let mut connection = PgConnection::connect_with(&db_config.without_db())
            .await
            .expect("Failed to connect to postgres");
        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, db_config.database_name).as_str())
            .await
            .expect("Failed to create database");
        let db_pool = PgPool::connect_with(db_config.with_db())
            .await
            .expect("Failed to connect to postgres");
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to migrate the database");
        db_pool
    }

    pub async fn get_route(&self, r: &str) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/{}", &self.address, r))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_hits(&self, url: &str) -> reqwest::Response {
	self.api_client
	    .get(&format!("{}/hits", &self.address))
	    .query(&[("url", url)])
	    .send()
	    .await
	    .expect("Failed to execute request")
    }

    pub async fn post_register(&self, body: &str) -> reqwest::Response {
	self.api_client
	    .post(&format!("{}/register", &self.address))
	    .header("Content-Type", "application/x-www-form-urlencoded")
	    .body(body.to_string())
	    .send()
	    .await
	    .expect("Failed to execute request")
    }

    pub async fn insert_page(&self) -> uuid::Uuid {
	let page_id = Uuid::new_v4();
	sqlx::query!(
	    r#"
INSERT INTO pages (page_id, owner, url)
VALUES ($1, $2, $3)"#,
	    page_id,
	    Uuid::new_v4(),
	    "https://example.com"
	)
	    .execute(&self.db)
	    .await
	    .expect("Failed to create some owner");
	page_id
    }
}
