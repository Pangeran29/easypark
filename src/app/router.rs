use axum::{routing::post, Router};
use sqlx::{Pool, Postgres};

use crate::error::aggregate::Error;

use super::auth::router::build as auth_router;
use super::file_upload::router::build as file_upload_router;
use super::parking_area::router::build as parking_lot_router;
use super::parking_history::router::build as parking_history_router;
use super::payment::router::build as payment_router;
use super::user::router::build as user_router;
use super::whatsapp::router::build as wa_router;

pub fn build(pool: Pool<Postgres>) -> Router {
    Router::new()
        .fallback(invalid_url_handler)
        .route("/", post(handler))
        .merge(auth_router(pool.clone()))
        .merge(user_router(pool.clone()))
        .merge(wa_router(pool.clone()))
        .merge(payment_router(pool.clone()))
        .merge(parking_lot_router(pool.clone()))
        .merge(parking_history_router(pool.clone()))
        .merge(file_upload_router())
}

async fn handler() {
    // tracing::debug!(?body, "handler received body");
}

async fn invalid_url_handler() -> Error {
    Error::NotFoundRejection(String::from("URL not found"))
}
