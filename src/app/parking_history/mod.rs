pub mod router;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{error::aggregate::Result as ResultApp, types::count::SqlxCount};

#[derive(Debug, Serialize)]
pub struct ParkingHistory {
    pub id: Uuid,
    pub ticket_status: TicketStatus,
    pub vehicle_type: VehicleType,
    pub payment: PaymentType,
    pub amount: f64,
    pub parking_lot_id: Uuid,
    pub easypark_id: Uuid,
    pub owner_id: Uuid,
    pub keeper_id: Uuid,
    pub transaction_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub check_in_date: Option<NaiveDateTime>,
    pub check_out_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct ParkingHistoryWithTotalAmount {
    pub id: Uuid,
    pub ticket_status: TicketStatus,
    pub vehicle_type: VehicleType,
    pub payment: PaymentType,
    pub amount: f64,
    pub total_amount: Option<f64>,
    pub parking_lot_id: Uuid,
    pub easypark_id: Uuid,
    pub owner_id: Uuid,
    pub keeper_id: Uuid,
    pub transaction_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub check_in_date: Option<NaiveDateTime>,
    pub check_out_date: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct HistoryFromQuery {
    pub id: Uuid,
    pub ticket_status: TicketStatus,
    pub vehicle_type: VehicleType,
    pub payment: PaymentType,
    pub amount: f64,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub area_name: String,
    pub address: String,
    pub image_url: String,
    pub forecast_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub check_in_date: Option<NaiveDateTime>,
    pub check_out_date: Option<NaiveDateTime>,
    pub easypark_name: String,
}

impl HistoryFromQuery {
    fn into_related_parking_history(self) -> RelatedParkingHistory {
        let created_at = match &self.created_at {
            Some(created_at) => {
                Some(TimeZone::from_utc_datetime(&Utc, created_at))
            },
            None => None,
        };

        let updated_at = match &self.updated_at {
            Some(updated_at) => {
                Some(TimeZone::from_utc_datetime(&Utc, updated_at))
            },
            None => None,
        };
        
        let check_in_date: Option<DateTime<Utc>> = match &self.check_in_date {
            Some(check_in_date) => {
                Some(TimeZone::from_utc_datetime(&Utc, check_in_date))
            },
            None => None,
        };

        let check_out_date = match &self.check_out_date {
            Some(check_out_date) => {
                Some(TimeZone::from_utc_datetime(&Utc, check_out_date))
            },
            None => None,
        };

        RelatedParkingHistory {
            id: self.id,
            ticket_status: self.ticket_status,
            vehicle_type: self.vehicle_type,
            payment: self.payment,
            total_amount: self.total_amount.unwrap_or(0.0),
            forecast_amount: self.forecast_amount.unwrap_or(0.0),
            amount: self.amount,
            created_at,
            updated_at,
            check_in_date,
            check_out_date,
            parking_lot: RelatedParkingLot {
                area_name: self.area_name,
                address: self.address,
                image_url: self.image_url,
            },
            easypark: RelatedEasypark {
                name: self.easypark_name
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RelatedParkingHistory {
    pub id: Uuid,
    pub ticket_status: TicketStatus,
    pub vehicle_type: VehicleType,
    pub payment: PaymentType,
    pub amount: f64,
    pub forecast_amount: f64,
    pub total_amount: f64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub check_in_date: Option<DateTime<Utc>>,
    pub check_out_date: Option<DateTime<Utc>>,
    pub parking_lot: RelatedParkingLot,
    pub easypark: RelatedEasypark
}

#[derive(Debug, Serialize)]
pub struct RelatedParkingLot {
    pub area_name: String,
    pub address: String,
    pub image_url: String,
}

#[derive(Debug, Serialize)]
pub struct RelatedEasypark {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AggregateQuery {
    pub payment_type: Option<PaymentType>,
    pub ticket_status: Option<TicketStatus>,
    pub easypark_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub keeper_id: Option<Uuid>,
    pub created_at_start_filter: Option<NaiveDateTime>,
    pub created_at_end_filter: Option<NaiveDateTime>,
    pub take: Option<i64>,
    pub skip: Option<i64>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct CalcQuery {
    pub owner_id: Option<Uuid>,
    pub keeper_id: Option<Uuid>,
    pub created_at_start_filter: NaiveDateTime,
    pub created_at_end_filter: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct ParkingHistoryCount {
    pub data: Option<i64>
}

#[derive(Debug, Serialize)]
pub struct MonthlyRecord {
    pub month: Option<String>,
    pub total_history: Option<i64>
}

#[derive(Debug, Serialize)]
pub struct CalcHistory {
    pub sum_all: Option<f64>,
    pub total_history: Option<i64>
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")] 
pub enum TicketStatus {
    Default,
    Active,
    NotActive
}

impl ParkingHistory {
    pub async fn save(self, pool: &Pool<Postgres>) -> ResultApp<ParkingHistory> {
        let data = sqlx::query_as!(
            ParkingHistory, 
            r#"
                insert into "parking_history" 
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) 
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
                    updated_at,
                    check_in_date,
                    check_out_date
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
            self.updated_at,
            self.check_in_date,
            self.check_out_date,
        )
            .fetch_one(pool)
            .await?;

        Ok(data)
    }

    pub async fn find_one(id: Uuid, pool: &Pool<Postgres>) -> ResultApp<ParkingHistory> {
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
                    updated_at,
                    check_in_date,
                    check_out_date
                from "parking_history" 
                where id = $1"#,  
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(data)
    }
    
    pub async fn update_transaction_id(old_transaction_id: Uuid, new_transaction_id: Uuid, pool: &Pool<Postgres>) -> ResultApp<ParkingHistoryWithTotalAmount> {
        let user = sqlx::query_as!(
            ParkingHistoryWithTotalAmount, 
            r#"
                with update_parking_history as (
                    update parking_history
                    set transaction_id = $2
                    where transaction_id = $1
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
                        updated_at,
                        check_in_date,
                        check_out_date,
                        ceil(extract(epoch from (now() - check_in_date)) / 3600) * amount as total_amount
                ),
                update_transaction as (
                    update transaction_history 
                    set id = $2
                    where id = $1
                )
                select * from update_parking_history
            "#,
            old_transaction_id,
            new_transaction_id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    async fn count(payload: AggregateQuery, pool: &Pool<Postgres>) -> ResultApp<SqlxCount> {
        let data = sqlx::query_as!(
            SqlxCount, 
            r#"
                select count(*) as data
                from parking_history ph
                join parking_lot pl on pl.id = ph.parking_lot_id
                where ($1::timestamp is null or ph.created_at >= $1) and ($2::timestamp is null or ph.created_at <= $2) and
                    (($3::uuid is not null and ph.easypark_id = $3) or ($4::uuid is not null and ph.owner_id = $4) or ($5::uuid is not null and ph.keeper_id = $5) or ($3::uuid is null and $4::uuid is null and $5::uuid is null)) and
                    ($6 = 'default' or ph.ticket_status = $6::ticket_status) and
                    ($7 = 'default' or ph.payment = $7::payment_type)
            "#,
            payload.created_at_start_filter,
            payload.created_at_end_filter,
            payload.easypark_id,
            payload.owner_id,
            payload.keeper_id,
            payload.ticket_status.unwrap_or(TicketStatus::Default) as TicketStatus,
            payload.payment_type.unwrap_or(PaymentType::Default) as PaymentType,
        )
            .fetch_one(pool)
            .await?;

        Ok(data)
    }

    async fn aggregate(payload: AggregateQuery, pool: &Pool<Postgres>) -> ResultApp<Vec<RelatedParkingHistory>> {
        let data = sqlx::query_as!(
            HistoryFromQuery, 
            r#"
                select ph.id, 
                    ph.ticket_status as "ticket_status!: TicketStatus", 
                    ph.vehicle_type as "vehicle_type!: VehicleType", 
                    ph.payment as "payment!: PaymentType", 
                    ph.amount,
                    ph.created_at, 
                    ph.updated_at,
                    pl.area_name,
                    pl.address,
                    pl.image_url,
                    CEIL(EXTRACT(EPOCH FROM (NOW() - ph.check_in_date)) / 3600) * ph.amount AS forecast_amount,
                    cast (tx.gross_amount as float) as total_amount,
                    ph.check_in_date, 
                    ph.check_out_date,
                    u."name" as easypark_name
                from parking_history ph
                join parking_lot pl on pl.id = ph.parking_lot_id
                join transaction_history tx on tx.id = ph.transaction_id
                join "user" u on u.id = ph.easypark_id
                where ($1::timestamp is null or ph.created_at >= $1) and ($2::timestamp is null or ph.created_at <= $2) and
                    (($3::uuid is not null and ph.easypark_id = $3) or ($4::uuid is not null and ph.owner_id = $4) or ($5::uuid is not null and ph.keeper_id = $5) or ($3::uuid is null and $4::uuid is null and $5::uuid is null)) and
                    ($6 = 'default' or ph.ticket_status = $6::ticket_status) and
                    ($7 = 'default' or ph.payment = $7::payment_type)
                limit $8
                offset $9
            "#,
            payload.created_at_start_filter,
            payload.created_at_end_filter,
            payload.easypark_id,
            payload.owner_id,
            payload.keeper_id,
            payload.ticket_status.unwrap_or(TicketStatus::Default) as TicketStatus,
            payload.payment_type.unwrap_or(PaymentType::Default) as PaymentType,
            payload.take,
            payload.skip,
        )
            .fetch_all(pool)
            .await?;

        let data: Vec<RelatedParkingHistory> = data.into_iter()
            .map(HistoryFromQuery::into_related_parking_history)
            .collect();

        Ok(data)
    }
    
    async fn find_active_ticket(easypark_id: Uuid, pool: &Pool<Postgres>) -> ResultApp<Vec<RelatedParkingHistory>> {
        let data = sqlx::query_as!(
            HistoryFromQuery, 
            r#"
                select ph.id, 
                    ph.ticket_status as "ticket_status!: TicketStatus", 
                    ph.vehicle_type as "vehicle_type!: VehicleType", 
                    ph.payment as "payment!: PaymentType", 
                    ph.amount,
                    ph.created_at, 
                    ph.updated_at,
                    pl.area_name,
                    pl.address,
                    pl.image_url,
                    CEIL(EXTRACT(EPOCH FROM (NOW() - ph.check_in_date)) / 3600) * ph.amount AS forecast_amount,
                    cast(tx.gross_amount as float) as total_amount,
                    ph.check_in_date,
                    ph.check_out_date,
                    u."name" as easypark_name
                from parking_history ph
                join parking_lot pl on pl.id = ph.parking_lot_id
                join transaction_history tx on tx.id = ph.transaction_id
                join "user" u on u.id = ph.easypark_id
                where ph.easypark_id = $1 and ph.ticket_status in ('active', 'default')
            "#,
            easypark_id
        )
            .fetch_all(pool)
            .await?;

        let data: Vec<RelatedParkingHistory> = data.into_iter()
            .map(HistoryFromQuery::into_related_parking_history)
            .collect();

        Ok(data)
    }
    
    async fn monthly_record(owner_id: Uuid, pool: &Pool<Postgres>) -> ResultApp<Vec<MonthlyRecord>> {
        let data = sqlx::query_as!(
            MonthlyRecord, 
            r#"
                SELECT
                    TO_CHAR(created_at, 'FMMonth') AS month,
                    COUNT(*) AS total_history
                FROM
                    parking_history
                WHERE owner_id = $1 and
                    EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM CURRENT_DATE)
                GROUP BY
                    month
                ORDER BY
                    MIN(created_at);
            "#,
            owner_id
        )
            .fetch_all(pool)
            .await?;

        Ok(data)
    }

    async fn filtered_calc(payload: CalcQuery, pool: &Pool<Postgres>) -> ResultApp<CalcHistory> {
        let data = sqlx::query_as!(
            CalcHistory, 
            r#"
                select sum(cast(th.gross_amount AS DOUBLE PRECISION)) as sum_all,
                    count(*) as total_history
                from parking_history ph
                join transaction_history th ON ph.transaction_id = th.id 
                where ticket_status = 'not_active' and 
                    ph.created_at >= $1 and 
                    ph.created_at <= $2 and 
                    ($3::uuid is null or ph.owner_id = $3) and
                    ($4::uuid is null or (ph.keeper_id = $4 and ph.payment = 'cash'))
            "#,
            payload.created_at_start_filter,
            payload.created_at_end_filter,
            payload.owner_id,
            payload.keeper_id,
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
    pub check_in_date: Option<NaiveDateTime>,
    pub check_out_date: Option<NaiveDateTime>,
}

impl UpdateParkingHistory {
    pub async fn update(self, id: Uuid, pool: &Pool<Postgres>) -> ResultApp<ParkingHistory> {
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
                    updated_at = coalesce($11, "parking_history".updated_at),
                    check_in_date = coalesce($12, "parking_history".check_in_date),                     
                    check_out_date = coalesce($13, "parking_history".check_out_date)
                where id = $14
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
                    updated_at,
                    check_in_date,
                    check_out_date
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
            self.check_in_date,
            self.check_out_date,
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn update_ticket_status(transaction_id: Uuid, status: TicketStatus, check_out_date: NaiveDateTime, pool: &Pool<Postgres>) -> ResultApp<ParkingHistory> {
        let user = sqlx::query_as!(
            ParkingHistory, 
            r#"
                update "parking_history" 
                set ticket_status = coalesce($1, "parking_history".ticket_status),
                    check_out_date = coalesce($2, "parking_history".check_out_date)
                where transaction_id = $3
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
                    updated_at,
                    check_in_date,
                    check_out_date
            "#,
            status as TicketStatus,
            check_out_date,
            transaction_id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}

