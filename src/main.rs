use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, MatchedPath, Request},
    Router,
};
use tokio::net::TcpListener;
use tower_http::{
    classify::ServerErrorsFailureClass, limit::RequestBodyLimitLayer, services::ServeDir,
    trace::TraceLayer,
};
use tracing::{error, info, info_span, Span};

mod app;
mod libs;

pub use self::libs::*;

#[tokio::main]
async fn main() {
    libs::trace::build();

    let pool = libs::database::build().await;

    let app = Router::new()
        .nest("/api", app::router::build(pool))
        .nest_service("/asset", ServeDir::new("./public/files"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|request: &Request<_>, _span: &Span| {
                    info!("Received {} request", request.method());
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        error!("Error: {}", error);
                    },
                ),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Server running on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
