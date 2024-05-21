use axum::{extract::State, middleware, routing::post, Router};
use base64::prelude::*;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use tracing::debug;
use uuid::Uuid;

use crate::app::user::User;
use crate::{
    error::aggregate::Result,
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/generate", post(generate))
        .route("/callback", post(callback))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/payment", router)
}

#[derive(Debug, Deserialize)]
struct TransactionPayload {
    user_phone: String,
    gross_amount: i32,
    item_name: String,
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct TransactionDetails {
    order_id: Uuid,
    gross_amount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ItemDetail {
    price: i32,
    quantity: i32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct CustomerDetails {
    first_name: String,
    phone: String,
}

#[derive(Serialize, Deserialize)]
struct Gopay {
    enable_callback: bool,
    callback_url: String,
}

#[derive(Serialize, Deserialize)]
struct PaymentData {
    payment_type: String,
    transaction_details: TransactionDetails,
    item_details: Vec<ItemDetail>,
    customer_details: CustomerDetails,
    gopay: Gopay,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    status_code: String,
    status_message: String,
    transaction_id: String,
    order_id: String,
    merchant_id: String,
    gross_amount: String,
    currency: String,
    payment_type: String,
    transaction_time: String,
    transaction_status: String,
    fraud_status: String,
    actions: Vec<Action>,
    expiry_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Action {
    name: String,
    method: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionCallback {
    pub transaction_time: String,
    pub transaction_status: String,
    pub transaction_id: String,
    pub status_message: String,
    pub status_code: String,
    pub signature_key: String,
    pub settlement_time: String,
    pub payment_type: String,
    pub order_id: String,
    pub merchant_id: String,
    pub gross_amount: String,
    pub fraud_status: String,
    pub currency: String,
}

async fn generate(
    State(pool): State<PgPool>,
    Body(payload): Body<TransactionPayload>,
) -> Result<AppSuccess<Transaction>> {
    let url = std::env::var("MIDTRANS_CHARGE_API").expect("MIDTRANS credential must be set");
    let server_key = std::env::var("MIDTRANS_SERVER_KEY").expect("MIDTRANS credential must be set");

    let api_key = BASE64_STANDARD.encode(format!("{}:", server_key));
    let api_key = format!("Basic {}", api_key);

    let TransactionPayload {
        user_phone,
        gross_amount,
        item_name,
    } = payload;

    let user = User::find_one(user_phone, &pool).await?;

    // Create the JSON body
    let payment_data = PaymentData {
        payment_type: "gopay".to_string(),
        transaction_details: TransactionDetails {
            order_id: Uuid::new_v4(),
            gross_amount,
        },
        item_details: vec![ItemDetail {
            price: gross_amount,
            quantity: 1,
            name: item_name,
        }],
        customer_details: CustomerDetails {
            first_name: user.name,
            phone: user.phone_number,
        },
        gopay: Gopay {
            enable_callback: true,
            callback_url: "https://ea0d-36-72-17-45.ngrok-free.app/api/payment/callback"
                .to_string(),
        },
    };

    // Serialize the data to JSON
    let json_body = serde_json::to_string(&payment_data).unwrap();

    // Create the reqwest client
    let client = Client::new();

    // Send the request
    let response = client
        .post(url)
        .header("accept", "application/json")
        .header("authorization", api_key)
        .header("content-type", "application/json")
        .body(json_body)
        .send()
        .await?;

    let data: Transaction = response.json().await?;

    Ok(AppSuccess(data))
}

async fn callback(
    State(_pool): State<PgPool>,
    Body(payload): Body<TransactionCallback>,
) -> Result<AppSuccess<()>> {
    debug!("HERE IS THE BODY FROM MIDTRANS: {:#?}", payload);
    Ok(AppSuccess(()))
}
