pub mod router;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error::aggregate::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionHistory {
    pub id: Uuid,
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

impl TransactionHistory {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<TransactionHistory> {
        let user = sqlx::query_as!(
            TransactionHistory,
            r#"
                insert into transaction_history (id, transaction_time, transaction_status, transaction_id, status_message, status_code, signature_key, settlement_time, payment_type, order_id, merchant_id, gross_amount, fraud_status, currency)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) 
                returning *
            "#,
            self.id,
            self.transaction_time,
            self.transaction_status,
            self.transaction_id,
            self.status_message,
            self.status_code,
            self.signature_key,
            self.settlement_time,
            self.payment_type,
            self.order_id,
            self.merchant_id,
            self.gross_amount,
            self.fraud_status,
            self.currency
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_one(id: Uuid, pool: &Pool<Postgres>) -> Result<TransactionHistory> {
        let user = sqlx::query_as!(
            TransactionHistory,
            r#"select * from transaction_history where id = $1"#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn update(self, pool: &Pool<Postgres>) -> Result<TransactionHistory> {
        let user = sqlx::query_as!(
            TransactionHistory,
            r#"
                update "transaction_history" 
                set transaction_time = coalesce($1, "transaction_history".transaction_time), 
                    transaction_status = coalesce($2, "transaction_history".transaction_status), 
                    transaction_id = coalesce($3, "transaction_history".transaction_id), 
                    status_message = coalesce($4, "transaction_history".status_message), 
                    status_code = coalesce($5, "transaction_history".status_code),
                    signature_key = coalesce($6, "transaction_history".signature_key),
                    settlement_time = coalesce($7, "transaction_history".settlement_time),
                    payment_type = coalesce($8, "transaction_history".payment_type),
                    order_id = coalesce($9, "transaction_history".order_id),
                    merchant_id = coalesce($10, "transaction_history".merchant_id),
                    gross_amount = coalesce($11, "transaction_history".gross_amount),
                    fraud_status = coalesce($12, "transaction_history".fraud_status),
                    currency = coalesce($13, "transaction_history".currency)
                where id = $14
                returning *
            "#,
            self.transaction_time,
            self.transaction_status,
            self.transaction_id,
            self.status_message,
            self.status_code,
            self.signature_key,
            self.settlement_time,
            self.payment_type,
            self.order_id,
            self.merchant_id,
            self.gross_amount,
            self.fraud_status,
            self.currency,
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}
