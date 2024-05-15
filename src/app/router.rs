use axum::{routing::post, Router};
use sqlx::{Pool, Postgres};

use crate::error::aggregate::Error;

use super::auth::router::build as auth_router;
use super::whatsapp::router::build as wa_router;

pub fn build(pool: Pool<Postgres>) -> Router {
    Router::new()
        .fallback(invalid_url_handler)
        .route("/", post(handler))
        .merge(auth_router(pool.clone()))
        .merge(wa_router(pool.clone()))
}

async fn handler() {
    // tracing::debug!(?body, "handler received body");
}

async fn invalid_url_handler() -> Error {
    Error::NotFoundRejection(String::from("URL not found"))
}
