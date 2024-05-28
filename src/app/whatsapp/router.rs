use std::collections::HashMap;

use axum::{extract::State, middleware, routing::post, Router};
use chrono::{DateTime, Datelike, Duration, Utc};
use rand::{rngs::OsRng, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};

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

    let account_sid = std::env::var("TWILIO_ACCOUNT_SID").expect("MIDTRANS credential must be set");
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").expect("MIDTRANS credential must be set");

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let to_number = format!("whatsapp:+{}", user.phone_number);
    let to_body = format!("Your OTP: {}", otp);

    let mut params = HashMap::new();
    params.insert("To", to_number.as_str());
    params.insert("From", "whatsapp:+14155238886");
    params.insert("Body", to_body.as_str());

    let client = Client::new();
    let response = client
        .post(&url)
        .basic_auth(account_sid, Some(auth_token))
        .form(&params)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Message sent successfully!");
    } else {
        eprintln!("Failed to send message: {:?}", response.text().await?);
    }

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
