use axum::{extract::State, middleware, routing::post, Router};
use base64::prelude::*;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::app::parking_history::ParkingHistory;
use crate::app::parking_history::TicketStatus;
use crate::app::parking_history::UpdateParkingHistory;
use crate::app::user::User;
use crate::extractor::current_user::authenticate_user;
use crate::{
    error::aggregate::Result,
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

use super::TransactionHistory;

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
    parking_history_id: Uuid,
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct TransactionDetails {
    order_id: Uuid,
    gross_amount: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ItemDetail {
    price: f64,
    quantity: u32,
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

async fn generate(
    State(pool): State<PgPool>,
    Body(payload): Body<TransactionPayload>,
) -> Result<AppSuccess<Transaction>> {
    let url = std::env::var("MIDTRANS_CHARGE_API").expect("MIDTRANS credential must be set");
    let server_key = std::env::var("MIDTRANS_SERVER_KEY").expect("MIDTRANS credential must be set");

    let api_key = BASE64_STANDARD.encode(format!("{}:", server_key));
    let api_key = format!("Basic {}", api_key);

    let TransactionPayload { parking_history_id } = payload;

    let parking_history = ParkingHistory::find_one(parking_history_id, &pool).await?;
    let easypark = User::find_one_by_id(parking_history.easypark_id, &pool).await?;

    let payment_data = PaymentData {
        payment_type: "gopay".to_string(),
        transaction_details: TransactionDetails {
            order_id: parking_history.transaction_id,
            gross_amount: parking_history.amount,
        },
        item_details: vec![ItemDetail {
            price: parking_history.amount,
            quantity: 1,
            name: "Parking Payment".to_string(),
        }],
        customer_details: CustomerDetails {
            first_name: easypark.name,
            phone: easypark.phone_number,
        },
        gopay: Gopay {
            enable_callback: true,
            callback_url: "https://ea0d-36-72-17-45.ngrok-free.app/api/payment/callback"
                .to_string(),
        },
    };

    let json_body = serde_json::to_string(&payment_data).unwrap();

    let client = Client::new();
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionCallback {
    pub transaction_time: Option<String>,
    pub transaction_status: Option<String>,
    pub transaction_id: Option<String>,
    pub status_message: Option<String>,
    pub status_code: Option<String>,
    pub signature_key: Option<String>,
    pub settlement_time: Option<String>,
    pub payment_type: Option<String>,
    pub order_id: Option<Uuid>,
    pub merchant_id: Option<String>,
    pub gross_amount: Option<String>,
    pub fraud_status: Option<String>,
    pub currency: Option<String>,
}

impl TransactionCallback {
    fn into_trasaction_history(self) -> TransactionHistory {
        TransactionHistory {
            id: self.order_id.unwrap(),
            transaction_time: self.transaction_time,
            transaction_status: self.transaction_status,
            transaction_id: self.transaction_id,
            status_message: self.status_message,
            status_code: self.status_code,
            signature_key: self.signature_key,
            settlement_time: self.settlement_time,
            payment_type: self.payment_type,
            order_id: self.order_id,
            merchant_id: self.merchant_id,
            gross_amount: self.gross_amount,
            fraud_status: self.fraud_status,
            currency: self.currency,
        }
    }
}

#[derive(Serialize)]
struct Callback {
    parking_history: ParkingHistory,
    transaction_history: TransactionHistory,
}

async fn callback(
    State(pool): State<PgPool>,
    Body(payload): Body<TransactionCallback>,
) -> Result<AppSuccess<Callback>> {
    let transaction_history = payload.into_trasaction_history();
    let transaction_history = transaction_history.update(&pool).await?;
    let parking_history = UpdateParkingHistory::update_ticket_status(
        transaction_history.id,
        TicketStatus::NotActive,
        &pool,
    )
    .await?;

    Ok(AppSuccess(Callback {
        parking_history,
        transaction_history,
    }))
}
