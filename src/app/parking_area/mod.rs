use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error::aggregate::Result;

pub mod router;

#[derive(Debug, Serialize)]
pub struct ParkingLot {
    pub id: Uuid,
    pub area_name: String,
    pub address: String,
    pub image_url: String,
    pub car_cost: f64,
    pub motor_cost: f64,
    pub owner_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl ParkingLot {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<ParkingLot> {
        let parking_lot = sqlx::query_as!(
            ParkingLot, 
            r#"insert into "parking_lot" values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id, area_name, address, image_url, car_cost, motor_cost, owner_id, created_at, updated_at"#,  
            self.id,
            self.area_name,
            self.address,
            self.image_url,
            self.car_cost,
            self.motor_cost,
            self.owner_id,
            self.created_at,
            self.updated_at
        )
            .fetch_one(pool)
            .await?;

        Ok(parking_lot)
    }

    pub async fn find_one(id: Uuid, pool: &Pool<Postgres>) -> Result<ParkingLot> {
        let parking_lot = sqlx::query_as!(
            ParkingLot, 
            r#"select * from parking_lot where id = $1"#,  
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(parking_lot)
    }
    
    pub async fn find_by_owner(owner_id: Uuid, pool: &Pool<Postgres>) -> Result<Vec<ParkingLot>> {
        let parking_lot = sqlx::query_as!(
            ParkingLot, 
            r#"select * from parking_lot where owner_id = $1"#,  
            owner_id
        )
            .fetch_all(pool)
            .await?;

        Ok(parking_lot)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateParkingLot {
    pub id: Option<Uuid>,
    pub area_name: Option<String>,
    pub address: Option<String>,
    pub image_url: Option<String>,
    pub car_cost: Option<f64>,
    pub motor_cost: Option<f64>,
    pub owner_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl UpdateParkingLot {
    pub async fn update(self, id: Uuid, pool: &Pool<Postgres>) -> Result<ParkingLot> {
        let user = sqlx::query_as!(
            ParkingLot, 
            r#"
                update "parking_lot" 
                    set area_name = coalesce($1, "parking_lot".area_name), 
                        address = coalesce($2, "parking_lot".address), 
                        image_url = coalesce($3, "parking_lot".image_url), 
                        car_cost = coalesce($4, "parking_lot".car_cost), 
                        motor_cost = coalesce($5, "parking_lot".motor_cost), 
                        owner_id = coalesce($6, "parking_lot".owner_id), 
                        created_at = coalesce($7, "parking_lot".created_at), 
                        updated_at = coalesce($8, "parking_lot".updated_at)
                    where id = $9
                    returning id, area_name, address, image_url, car_cost, motor_cost, owner_id, created_at, updated_at
            "#,  
            self.area_name,
            self.address,
            self.image_url,
            self.car_cost,
            self.motor_cost,
            self.owner_id,
            self.created_at,
            self.updated_at,
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}

