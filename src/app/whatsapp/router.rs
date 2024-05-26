use axum::{extract::State, middleware, routing::post, Router};
use chrono::{DateTime, Datelike, Duration, Utc};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use tracing::debug;

use crate::{
    app::user::User,
    error::aggregate::Result,
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/send", post(send))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/whatsapp", router)
}

#[derive(Debug, Deserialize)]
struct SendWhatsappPayload {
    phone: String,
}

#[derive(Debug, Serialize)]
struct SendWaResponse {
    message: String,
    expires_in: DateTime<Utc>,
}

async fn send(
    State(pool): State<PgPool>,
    Body(payload): Body<SendWhatsappPayload>,
) -> Result<AppSuccess<SendWaResponse>> {
    let SendWhatsappPayload { phone } = payload;
    let user = User::find_one(phone, &pool).await?;

    let mut rng = OsRng;
    let otp: i32 = rng.gen_range(100000..1000000);

    let text = format!("Here+is+your+OTP+{}", otp);
    let url = format!(
        "https://api.callmebot.com/whatsapp.php?phone={}&text={}&apikey=7839565",
        user.phone_number, text
    );

    let body = reqwest::get(url).await?.text().await?;
    debug!("Whatsapp Reqwest : {}", body);

    let now = Utc::now();
    let exp = now
        .with_year(now.year() + 1)
        .unwrap_or_else(|| now + Duration::days(365));

    user.update_otp(otp, &pool).await?;

    Ok(AppSuccess(SendWaResponse {
        message: "Success to send SMS to given phone number".to_string(),
        expires_in: exp,
    }))
}
