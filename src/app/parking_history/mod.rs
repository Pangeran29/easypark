use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error::aggregate::Result;

pub mod router;

#[derive(Debug, Serialize)]
pub struct ParkingHistory {
    pub id: Uuid,
    pub ticket_status: TicketStatus,
    pub vehicle_type: VehicleType,
    pub payment: PaymentType,
    pub amount: f64,
    pub parking_lot_id: Uuid,
    pub easypark_id: Uuid,
    pub keeper_id: Uuid,
    pub owner_id: Uuid,
    pub transaction_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "vehicle_type", rename_all = "snake_case")] 
pub enum VehicleType {
    Default,
    Car,
    Motor
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "payment_type", rename_all = "snake_case")] 
pub enum PaymentType {
    Default,
    Cash,
    Qr
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")] 
pub enum TicketStatus {
    Default,
    Active,
    NotActive
}


impl ParkingHistory {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<ParkingHistory> {
        let data = sqlx::query_as!(
            ParkingHistory, 
            r#"
                insert into "parking_history" 
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
                returning id, 
                    ticket_status as "ticket_status!: TicketStatus", 
                    vehicle_type as "vehicle_type!: VehicleType", 
                    payment as "payment!: PaymentType", 
                    amount,
                    parking_lot_id,
                    easypark_id,
                    keeper_id,
                    owner_id,
                    transaction_id,
                    created_at, 
                    updated_at
            "#,  
            self.id,
            self.ticket_status as TicketStatus,
            self.vehicle_type as VehicleType,
            self.payment as PaymentType,
            self.amount,
            self.parking_lot_id,
            self.easypark_id,
            self.keeper_id,
            self.owner_id,
            self.transaction_id,
            self.created_at,
            self.updated_at
        )
            .fetch_one(pool)
            .await?;

        Ok(data)
    }

    pub async fn find_one(id: Uuid, pool: &Pool<Postgres>) -> Result<ParkingHistory> {
        let data = sqlx::query_as!(
            ParkingHistory, 
            r#"
                select id, 
                    ticket_status as "ticket_status!: TicketStatus", 
                    vehicle_type as "vehicle_type!: VehicleType", 
                    payment as "payment!: PaymentType", 
                    amount,
                    parking_lot_id,
                    easypark_id,
                    keeper_id,
                    owner_id,
                    transaction_id,
                    created_at, 
                    updated_at
                from "parking_history" 
                where id = $1"#,  
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(data)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateParkingHistory {
    pub id: Option<Uuid>,
    pub ticket_status: Option<TicketStatus>,
    pub vehicle_type: Option<VehicleType>,
    pub payment: Option<PaymentType>,
    pub amount: Option<f64>,
    pub parking_lot_id: Option<Uuid>,
    pub easypark_id: Option<Uuid>,
    pub keeper_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl UpdateParkingHistory {
    pub async fn update(self, id: Uuid, pool: &Pool<Postgres>) -> Result<ParkingHistory> {
        let user = sqlx::query_as!(
            ParkingHistory, 
            r#"
                update "parking_history" 
                set ticket_status = coalesce($1, "parking_history".ticket_status), 
                    vehicle_type = coalesce($2, "parking_history".vehicle_type),                     
                    payment = coalesce($3, "parking_history".payment),                     
                    amount = coalesce($4, "parking_history".amount),                     
                    parking_lot_id = coalesce($5, "parking_history".parking_lot_id),                     
                    easypark_id = coalesce($6, "parking_history".easypark_id),                     
                    keeper_id = coalesce($7, "parking_history".keeper_id),                     
                    owner_id = coalesce($8, "parking_history".owner_id),                     
                    transaction_id = coalesce($9, "parking_history".transaction_id),                     
                    created_at = coalesce($10, "parking_history".created_at),                     
                    updated_at = coalesce($11, "parking_history".updated_at)
                where id = $12
                returning id, 
                    ticket_status as "ticket_status!: TicketStatus", 
                    vehicle_type as "vehicle_type!: VehicleType", 
                    payment as "payment!: PaymentType", 
                    amount,
                    parking_lot_id,
                    easypark_id,
                    keeper_id,
                    owner_id,
                    transaction_id,
                    created_at, 
                    updated_at
            "#,
            self.ticket_status.unwrap_or(TicketStatus::Default) as TicketStatus,
            self.vehicle_type.unwrap_or(VehicleType::Default) as VehicleType,
            self.payment.unwrap_or(PaymentType::Default) as PaymentType,
            self.amount,
            self.parking_lot_id,
            self.easypark_id,
            self.keeper_id,
            self.owner_id,
            self.transaction_id,
            self.created_at,
            self.updated_at,
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn update_ticket_status(transaction_id: Uuid, status: TicketStatus, pool: &Pool<Postgres>) -> Result<ParkingHistory> {
        let user = sqlx::query_as!(
            ParkingHistory, 
            r#"
                update "parking_history" 
                set ticket_status = coalesce($1, "parking_history".ticket_status)
                where transaction_id = $2
                returning id, 
                    ticket_status as "ticket_status!: TicketStatus", 
                    vehicle_type as "vehicle_type!: VehicleType", 
                    payment as "payment!: PaymentType", 
                    amount,
                    parking_lot_id,
                    easypark_id,
                    keeper_id,
                    owner_id,
                    transaction_id,
                    created_at, 
                    updated_at
            "#,
            status as TicketStatus,
            transaction_id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}

