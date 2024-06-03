use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, patch, post},
    Router,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
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
    AggregateQuery, CalcHistory, CalcQuery, MonthlyRecord, ParkingHistory, PaymentType,
    RelatedParkingHistory, TicketStatus, UpdateParkingHistory, VehicleType,
};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/", post(create))
        .route("/:id", patch(update).get(detail))
        .route("/aggregate", get(aggregate))
        .route("/active-ticket/:id", get(get_active_ticket))
        .route("/monthly", get(get_monthly_history))
        .route("/filtered-calc", get(get_filtered_calc))
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
    check_in_date: Option<NaiveDateTime>,
    check_out_date: Option<NaiveDateTime>,
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
            check_in_date: None,
            check_out_date: None,
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

    let related_history = ParkingHistory::find_active_ticket(easypark.id, &pool).await?;

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
    ticket_status: Option<TicketStatus>,
    vehicle_type: Option<VehicleType>,
    payment: Option<PaymentType>,
    parking_lot_id: Option<Uuid>,
    easypark_id: Option<Uuid>,
    keeper_id: Option<Uuid>,
    owner_id: Option<Uuid>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    check_in_date: Option<NaiveDateTime>,
    check_out_date: Option<NaiveDateTime>,
}

impl UpdateParkingHistoryPayload {
    fn into_update_parking_history(self) -> UpdateParkingHistory {
        UpdateParkingHistory {
            id: None,
            ticket_status: self.ticket_status,
            vehicle_type: self.vehicle_type,
            payment: self.payment,
            amount: None,
            parking_lot_id: self.parking_lot_id,
            easypark_id: self.easypark_id,
            keeper_id: self.keeper_id,
            owner_id: self.owner_id,
            transaction_id: None,
            created_at: None,
            updated_at: Some(Utc::now().naive_utc()),
            check_in_date: None,
            check_out_date: None,
        }
    }
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Body(payload): Body<UpdateParkingHistoryPayload>,
) -> Result<AppSuccess<History>> {
    let mut parking_history = payload.into_update_parking_history();

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

    match parking_history.ticket_status {
        Some(status) => match status {
            TicketStatus::Default => {}
            TicketStatus::Active => parking_history.check_in_date = Some(Utc::now().naive_utc()),
            TicketStatus::NotActive => {}
        },
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

async fn get_active_ticket(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<AppSuccess<RelatedParkingHistory>> {
    let mut active_ticket = ParkingHistory::find_active_ticket(id, &pool).await?;
    if active_ticket.len() < 1 {
        return Err(Error::BadRequest("Ticket is not issue".to_string()));
    }
    let active_ticket = active_ticket.remove(0);
    Ok(AppSuccess(active_ticket))
}

#[derive(Deserialize, Clone, Serialize)]
pub struct MonthlyHistory {
    pub owner_id: Uuid,
}

async fn get_monthly_history(
    State(pool): State<PgPool>,
    Query(payload): Query<MonthlyHistory>,
) -> Result<AppSuccess<Vec<MonthlyRecord>>> {
    let monthly_record = ParkingHistory::monthly_record(payload.owner_id, &pool).await?;
    Ok(AppSuccess(monthly_record))
}

#[derive(Deserialize, Clone, Serialize)]
pub struct CalcPayload {
    pub owner_id: Option<Uuid>,
    pub keeper_id: Option<Uuid>,
    pub created_at_start_filter: DateTime<Utc>,
    pub created_at_end_filter: DateTime<Utc>,
}

impl CalcPayload {
    fn into_calc_query(self) -> CalcQuery {
        let created_at_end_filter = self.created_at_end_filter.naive_utc();
        let created_at_start_filter = self.created_at_start_filter.naive_utc();

        CalcQuery {
            owner_id: self.owner_id,
            keeper_id: self.keeper_id,
            created_at_end_filter,
            created_at_start_filter,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FilteredCalc {
    pub sum_all: f64,
    pub total_history: i64
}

async fn get_filtered_calc(
    State(pool): State<PgPool>,
    Query(payload): Query<CalcPayload>,
) -> Result<AppSuccess<FilteredCalc>> {
    let query = payload.into_calc_query();
    let filtered_calc = ParkingHistory::filtered_calc(query, &pool).await?;
    Ok(AppSuccess(FilteredCalc {
        sum_all: filtered_calc.sum_all.unwrap_or(0.0),
        total_history: filtered_calc.total_history.unwrap_or(0)
    }))
}
