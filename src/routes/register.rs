use actix_web::{web, Responder};
use sqlx::PgPool;
use serde::Deserialize;

pub async fn register(
    form: web::Form<RegisterPageForm>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    format!("{}", form.into_inner().url)
}

#[derive(Deserialize)]
pub struct RegisterPageForm {
    url: String,
}
