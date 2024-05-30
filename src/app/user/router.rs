use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, patch, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::{
    error::aggregate::Result,
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

use super::{Role, UpdateUser, User, UserStatus};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/", post(create).get(aggregate))
        .route("/aggregate", get(aggregate))
        .route("/:phone_number", patch(update).get(get_by_phone_number))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/user", router)
}

async fn get_by_phone_number(
    State(pool): State<PgPool>,
    Path(phone_number): Path<String>,
) -> Result<AppSuccess<User>> {
    let user = User::find_one(phone_number, &pool).await?;
    Ok(AppSuccess(user))
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserPayload {
    phone_number: String,
    name: String,
    nik: String,
    role: Role,
    status: UserStatus,
    otp: Option<i32>,
    belong_to_parking_lot_id: Option<Uuid>,
    owner_id: Option<Uuid>,
}

impl CreateUserPayload {
    pub fn into_user(self) -> User {
        User {
            id: Uuid::new_v4(),
            phone_number: self.phone_number,
            name: self.name,
            nik: self.nik,
            role: self.role,
            status: self.status,
            otp: None,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: None,
            parking_lot_id: self.belong_to_parking_lot_id,
            owner_id: self.owner_id,
        }
    }
}

async fn create(
    State(pool): State<PgPool>,
    Body(payload): Body<CreateUserPayload>,
) -> Result<AppSuccess<User>> {
    let user = payload.into_user();
    let user = user.save(&pool).await?;
    Ok(AppSuccess(user))
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateUserPayload {
    phone_number: Option<String>,
    name: Option<String>,
    nik: Option<String>,
    role: Option<Role>,
    status: Option<UserStatus>,
    otp: Option<i32>,
    belong_to_parking_lot_id: Option<Uuid>,
    owner_id: Option<Uuid>,
}

impl UpdateUserPayload {
    pub fn into_update_user(self) -> UpdateUser {
        UpdateUser {
            id: None,
            phone_number: self.phone_number,
            name: self.name,
            nik: self.nik,
            role: self.role,
            status: self.status,
            otp: self.otp,
            created_at: None,
            updated_at: Some(Utc::now().naive_utc()),
            parking_lot_id: self.belong_to_parking_lot_id,
            owner_id: self.owner_id,
        }
    }
}

async fn update(
    State(pool): State<PgPool>,
    Path(phone_number): Path<String>,
    Body(payload): Body<UpdateUserPayload>,
) -> Result<AppSuccess<User>> {
    let user = payload.into_update_user();
    let user = user.update(phone_number, &pool).await?;
    Ok(AppSuccess(user))
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct UserAggregatePayload {
    pub belong_to_parking_lot_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub take: Option<i64>,
    pub skip: Option<i64>,
}

#[derive(Serialize)]
struct Aggregate {
    meta: MetaAggregate,
    user: Vec<User>,
}

#[derive(Serialize)]
struct MetaAggregate {
    total_data: i64,
    query: UserAggregatePayload,
}

async fn aggregate(
    State(pool): State<PgPool>,
    Query(payload): Query<UserAggregatePayload>,
) -> Result<AppSuccess<Aggregate>> {
    let count = User::count(&pool, payload).await?;
    let user = User::aggregate(&pool, payload).await?;
    Ok(AppSuccess(Aggregate {
        meta: MetaAggregate {
            total_data: count.data.unwrap_or(0),
            query: payload,
        },
        user,
    }))
}
