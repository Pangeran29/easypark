use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, patch, post},
    Router,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use tracing::debug;
use uuid::Uuid;

use crate::{
    app::{
        parking_area::ParkingLot,
        payment::TransactionHistory,
        user::{Role, User},
    },
    error::aggregate::{Error, Result},
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

use super::{
    AggregateQuery, ParkingHistory, PaymentType, RelatedParkingHistory, TicketStatus,
    UpdateParkingHistory, VehicleType,
};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/", post(create))
        .route("/:id", patch(update).get(detail))
        .route("/aggregate", get(aggregate))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/parking-history", router)
}

#[derive(Serialize, Deserialize)]
struct CreateParkingHistoryPayload {
    vehicle_type: VehicleType,
    payment: PaymentType,
    parking_lot_id: Uuid,
    easypark_id: Uuid,
    keeper_id: Uuid,
    transaction_id: Option<Uuid>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl CreateParkingHistoryPayload {
    fn into_parking_history(self) -> ParkingHistory {
        ParkingHistory {
            id: Uuid::new_v4(),
            ticket_status: TicketStatus::Default,
            vehicle_type: self.vehicle_type,
            payment: self.payment,
            amount: 0.0,
            parking_lot_id: self.parking_lot_id,
            easypark_id: self.easypark_id,
            keeper_id: self.keeper_id,
            owner_id: Uuid::new_v4(),
            transaction_id: Uuid::new_v4(),
            created_at: Some(Utc::now().naive_utc()),
            updated_at: None,
        }
    }
}

#[derive(Serialize)]
struct History {
    parking_history: ParkingHistory,
    transaction_history: TransactionHistory,
}

async fn create(
    State(pool): State<PgPool>,
    Body(payload): Body<CreateParkingHistoryPayload>,
) -> Result<AppSuccess<History>> {
    let mut parking_history = payload.into_parking_history();

    let easypark = User::find_one_by_id(parking_history.easypark_id, &pool).await?;
    if easypark.role != Role::Easypark {
        return Err(Error::BadRequest(
            "Provided easypark id is not having Easypark role".to_string(),
        ));
    }

    let aggregate_payload = AggregatePayload {
        payment_type: None,
        ticket_status: Some(TicketStatus::Active),
        easypark_id: Some(easypark.id),
        owner_id: None,
        keeper_id: None,
        created_at_start_filter: None,
        created_at_end_filter: None,
        take: None,
        skip: None,
    };

    let related_history =
        ParkingHistory::aggregate(aggregate_payload.into_aggregate_query(), &pool).await?;
    
    if related_history.len() >= 1 {
        return Err(Error::BadRequest("Ticket already issue".to_string()));
    }

    let keeper = User::find_one_by_id(parking_history.keeper_id, &pool).await?;
    if keeper.role != Role::ParkKeeper {
        return Err(Error::BadRequest(
            "Provided keeper id is not having ParkKeeper role".to_string(),
        ));
    }

    let parking_lot = ParkingLot::find_one(parking_history.parking_lot_id, &pool).await?;
    if keeper.parking_lot_id != Some(parking_lot.id) {
        return Err(Error::BadRequest(
            "Provided keeper is not belong to provided parking keeper".to_string(),
        ));
    }

    parking_history.amount = match parking_history.vehicle_type {
        VehicleType::Default => parking_lot.car_cost,
        VehicleType::Car => parking_lot.car_cost,
        VehicleType::Motor => parking_lot.motor_cost,
    };

    let transaction_history = TransactionHistory {
        id: parking_history.transaction_id,
        transaction_time: None,
        transaction_status: None,
        transaction_id: None,
        status_message: None,
        status_code: None,
        signature_key: None,
        settlement_time: None,
        payment_type: None,
        order_id: None,
        merchant_id: None,
        gross_amount: None,
        fraud_status: None,
        currency: None,
    };

    let transaction_history = transaction_history.save(&pool).await?;

    parking_history.transaction_id = transaction_history.id;
    parking_history.owner_id = parking_lot.owner_id;

    let parking_history = parking_history.save(&pool).await?;

    Ok(AppSuccess(History {
        parking_history,
        transaction_history,
    }))
}

#[derive(Serialize, Deserialize)]
struct UpdateParkingHistoryPayload {
    ticket_status: TicketStatus,
    vehicle_type: VehicleType,
    payment: PaymentType,
    parking_lot_id: Uuid,
    easypark_id: Uuid,
    keeper_id: Uuid,
    owner_id: Uuid,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl UpdateParkingHistoryPayload {
    fn into_update_parking_history(self) -> UpdateParkingHistory {
        UpdateParkingHistory {
            id: None,
            ticket_status: Some(self.ticket_status),
            vehicle_type: Some(self.vehicle_type),
            payment: Some(self.payment),
            amount: None,
            parking_lot_id: Some(self.parking_lot_id),
            easypark_id: Some(self.easypark_id),
            keeper_id: Some(self.keeper_id),
            owner_id: Some(self.owner_id),
            transaction_id: None,
            created_at: None,
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Body(payload): Body<UpdateParkingHistoryPayload>,
) -> Result<AppSuccess<History>> {
    let parking_history = payload.into_update_parking_history();

    match &parking_history.easypark_id {
        Some(id) => {
            let easypark = User::find_one_by_id(*id, &pool).await?;
            if easypark.role != Role::Easypark {
                return Err(Error::BadRequest(
                    "Provided easypark id is not having Easypark role".to_string(),
                ));
            }
        }
        None => {}
    }

    match &parking_history.keeper_id {
        Some(id) => {
            let keeper = User::find_one_by_id(*id, &pool).await?;
            if keeper.role != Role::ParkKeeper {
                return Err(Error::BadRequest(
                    "Provided keeper id is not having ParkKeeper role".to_string(),
                ));
            }
        }
        None => {}
    }

    match &parking_history.owner_id {
        Some(owner_id) => {
            let owner = User::find_one_by_id(*owner_id, &pool).await?;
            if owner.role != Role::ParkOwner {
                return Err(Error::BadRequest(
                    "Provided owner id is not having ParkOwner role".to_string(),
                ));
            }
        }
        None => {}
    }

    match &parking_history.parking_lot_id {
        Some(parking_lot_id) => {
            let parking_lot = ParkingLot::find_one(*parking_lot_id, &pool).await?;
            match &parking_history.owner_id {
                Some(owner_id) => {
                    if parking_lot.owner_id != *owner_id {
                        return Err(Error::BadRequest(
                            "Parking lot is not owned by the provided owner id".to_string(),
                        ));
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    let parking_history = parking_history.update(id, &pool).await?;
    let transaction_history =
        TransactionHistory::find_one(parking_history.transaction_id, &pool).await?;

    Ok(AppSuccess(History {
        parking_history,
        transaction_history,
    }))
}

#[derive(Serialize)]
struct DetailParkingHistory {
    parking_history: ParkingHistory,
    easypark: User,
    keeper: User,
    owner: User,
    parking_lot: ParkingLot,
    transaction: TransactionHistory,
}

async fn detail(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<AppSuccess<DetailParkingHistory>> {
    let parking_history = ParkingHistory::find_one(id, &pool).await?;
    let easypark = User::find_one_by_id(parking_history.easypark_id, &pool).await?;
    let keeper = User::find_one_by_id(parking_history.keeper_id, &pool).await?;
    let owner = User::find_one_by_id(parking_history.owner_id, &pool).await?;
    let parking_lot = ParkingLot::find_one(parking_history.parking_lot_id, &pool).await?;
    let transaction = TransactionHistory::find_one(parking_history.transaction_id, &pool).await?;
    Ok(AppSuccess(DetailParkingHistory {
        parking_history,
        easypark,
        keeper,
        owner,
        parking_lot,
        transaction,
    }))
}

#[derive(Deserialize, Clone, Serialize)]
pub struct AggregatePayload {
    pub payment_type: Option<PaymentType>,
    pub ticket_status: Option<TicketStatus>,
    pub easypark_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub keeper_id: Option<Uuid>,
    pub created_at_start_filter: Option<DateTime<Utc>>,
    pub created_at_end_filter: Option<DateTime<Utc>>,
    pub take: Option<i64>,
    pub skip: Option<i64>,
}

impl AggregatePayload {
    fn into_aggregate_query(self) -> AggregateQuery {
        let created_at_start_filter = match self.created_at_start_filter {
            Some(date) => Some(date.naive_utc()),
            None => None,
        };

        let created_at_end_filter = match self.created_at_end_filter {
            Some(date) => Some(date.naive_utc()),
            None => None,
        };

        AggregateQuery {
            payment_type: self.payment_type,
            ticket_status: self.ticket_status,
            easypark_id: self.easypark_id,
            owner_id: self.owner_id,
            keeper_id: self.keeper_id,
            created_at_start_filter,
            created_at_end_filter,
            take: self.take,
            skip: self.skip,
        }
    }
}

#[derive(Serialize)]
struct Aggregate {
    meta: MetaAggregate,
    parking_history: Vec<RelatedParkingHistory>,
}

#[derive(Serialize)]
struct MetaAggregate {
    total_data: i64,
    query: AggregatePayload,
}

async fn aggregate(
    State(pool): State<PgPool>,
    Query(payload): Query<AggregatePayload>,
) -> Result<AppSuccess<Aggregate>> {
    let query = payload.clone().into_aggregate_query();

    let count = ParkingHistory::count(query.clone(), &pool).await?;
    let parking_history = ParkingHistory::aggregate(query.clone(), &pool).await?;

    Ok(AppSuccess(Aggregate {
        meta: MetaAggregate {
            total_data: count.data.unwrap_or(0),
            query: payload,
        },
        parking_history,
    }))
}
