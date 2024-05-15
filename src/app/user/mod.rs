use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{ Pool, Postgres};
use uuid::Uuid;

use crate::error::aggregate::Result;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub phone: String,
    pub name: String,
    pub nik: String,
    pub role: String,
    pub status: String,
    pub otp: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl User {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"insert into users (id, phone, name, nik, role, status, otp, created_at, updated_at) values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id, phone, name, nik, role, status, otp, created_at, updated_at"#,  
            self.id,
            self.phone,
            self.name,
            self.nik,
            self.role,
            self.status,
            self.otp,
            self.created_at,
            self.updated_at
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn find_one(phone: String, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"select * from users where phone = $1"#,
            phone
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn update_otp(self, otp: i32, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"update users set otp = $1 where phone = $2 returning *"#,
            otp,
            self.phone
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}
