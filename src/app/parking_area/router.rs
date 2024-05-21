use std::path::Path as StdPath;

use axum::{
    extract::{Path, State},
    middleware,
    routing::{patch, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

use crate::{
    app::user::{Role, User},
    error::aggregate::{Error, Result},
    extractor::{app_body::Body, app_json::AppSuccess},
    middleware::base::print_request_body,
};

use super::{ParkingLot, UpdateParkingLot};

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/", post(create))
        .route("/:id", patch(update).get(detail))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/parking-lot", router)
}

#[derive(Serialize, Deserialize)]
struct CreateParkingLotPayload {
    area_name: String,
    address: String,
    file_name: String,
    car_cost: f64,
    motor_cost: f64,
    owner_id: Uuid,
}

impl CreateParkingLotPayload {
    fn into_parking_lot(self) -> ParkingLot {
        ParkingLot {
            id: Uuid::new_v4(),
            area_name: self.area_name,
            address: self.address,
            image_url: self.file_name,
            car_cost: self.car_cost,
            motor_cost: self.motor_cost,
            owner_id: self.owner_id,
            created_at: Some(Utc::now().naive_utc()),
            updated_at: None,
        }
    }
}

async fn create(
    State(pool): State<PgPool>,
    Body(payload): Body<CreateParkingLotPayload>,
) -> Result<AppSuccess<ParkingLot>> {
    let user = User::find_one_by_id(payload.owner_id, &pool).await?;
    if user.role != Role::ParkOwner {
        return Err(Error::BadRequest(
            "Related user is not having owner role".to_string(),
        ));
    }

    let dir = format!("./public/files/{}", payload.file_name);
    let path = StdPath::new(&dir);
    if !path.exists() {
        return Err(Error::BadRequest("Image not found".to_string()));
    }

    let parking_lot = payload.into_parking_lot();
    let parking_lot = parking_lot.save(&pool).await?;
    Ok(AppSuccess(parking_lot))
}

#[derive(Serialize, Deserialize)]
struct UpdateParkingLotPayload {
    area_name: Option<String>,
    address: Option<String>,
    file_name: Option<String>,
    car_cost: Option<f64>,
    motor_cost: Option<f64>,
    owner_id: Option<Uuid>,
}

impl UpdateParkingLotPayload {
    fn into_update_parking_lot(self) -> UpdateParkingLot {
        UpdateParkingLot {
            id: None,
            area_name: self.area_name,
            address: self.address,
            image_url: self.file_name,
            car_cost: self.car_cost,
            motor_cost: self.motor_cost,
            owner_id: self.owner_id,
            created_at: None,
            updated_at: Some(Utc::now().naive_utc()),
        }
    }
}

async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Body(payload): Body<UpdateParkingLotPayload>,
) -> Result<AppSuccess<ParkingLot>> {
    match &payload.owner_id {
        Some(owner_id) => {
            let user = User::find_one_by_id(*owner_id, &pool).await?;
            if user.role != Role::ParkOwner {
                return Err(Error::BadRequest(
                    "Related user is not having owner role".to_string(),
                ));
            }
        }
        None => {}
    }

    match &payload.file_name {
        Some(file_name) => {
            let dir = format!("./public/files/{}", *file_name);
            let path = StdPath::new(&dir);
            if !path.exists() {
                return Err(Error::BadRequest("Image not found".to_string()));
            }
        }
        None => {}
    }

    let parking_lot = payload.into_update_parking_lot();
    let parking_lot = parking_lot.update(id, &pool).await?;
    Ok(AppSuccess(parking_lot))
}

async fn detail(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<AppSuccess<ParkingLot>> {
    let parking_lot = ParkingLot::find_one(id, &pool).await?;
    Ok(AppSuccess(parking_lot))
}
