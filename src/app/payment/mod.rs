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
    pub order_id: Option<String>,
    pub merchant_id: Option<String>,
    pub gross_amount: Option<String>,
    pub fraud_status: Option<String>,
    pub currency: Option<String>,
}

impl TransactionHistory {
    pub async fn _save(self, pool: &Pool<Postgres>) -> Result<TransactionHistory> {
        let user = sqlx::query_as!(
            TransactionHistory, 
            r#"insert into transaction_history (id, transaction_time, transaction_status, transaction_id, status_message, status_code, signature_key, settlement_time, payment_type, order_id, merchant_id, gross_amount, fraud_status, currency) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) returning *"#,  
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
    
    // pub async fn find_one(phone: String, pool: &Pool<Postgres>) -> Result<User> {
    //     let user = sqlx::query_as!(
    //         User, 
    //         r#"select * from users where phone = $1"#,
    //         phone
    //     )
    //         .fetch_one(pool)
    //         .await?;

    //     Ok(user)
    // }
    
    // pub async fn update_otp(self, otp: i32, pool: &Pool<Postgres>) -> Result<User> {
    //     let user = sqlx::query_as!(
    //         User, 
    //         r#"update users set otp = $1 where phone = $2 returning *"#,
    //         otp,
    //         self.phone
    //     )
    //         .fetch_one(pool)
    //         .await?;

    //     Ok(user)
    // }
}
