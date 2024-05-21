pub mod router;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::error::aggregate::Result;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub phone_number: String,
    pub name: String,
    pub nik: String,
    pub role: Role,
    pub status: UserStatus,
    pub otp: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "role", rename_all = "snake_case")] 
pub enum Role {
    Default,
    Easypark,
    ParkKeeper,
    ParkOwner,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")] 
pub enum UserStatus {
    Default,
    Active,
    NotActive
}

impl User {
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"insert into "user" values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id, phone_number, name, nik, role as "role!: Role", status as "status!: UserStatus", otp, created_at, updated_at"#,  
            self.id,
            self.phone_number,
            self.name,
            self.nik,
            self.role as Role,
            self.status as UserStatus,
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
            r#"select id, phone_number, name, nik, role as "role!: Role", status as "status!:UserStatus", otp, created_at, updated_at  from "user" where phone_number = $1"#,
            phone
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn find_one_by_id(id: Uuid, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"select id, phone_number, name, nik, role as "role!: Role", status as "status!:UserStatus", otp, created_at, updated_at  from "user" where id = $1"#,
            id
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn update_otp(self, otp: i32, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"update "user" set otp = $1 where phone_number = $2 returning id, phone_number, name, nik, role as "role!: Role", status as "status!:UserStatus", otp, created_at, updated_at"#,
            otp,
            self.phone_number
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn update_status(self, status: UserStatus, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"update "user" set status = $1 where phone_number = $2 returning id, phone_number, name, nik, role as "role!: Role", status as "status!:UserStatus", otp, created_at, updated_at"#,
            status as UserStatus,
            self.phone_number
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
    
    pub async fn invalidate_otp(self, pool: &Pool<Postgres>) -> Result<User> {
        let otp: Option<i32> = None;
        let user = sqlx::query_as!(
            User, 
            r#"update "user" set otp = $1 where phone_number = $2 returning id, phone_number, name, nik, role as "role!: Role", status as "status!:UserStatus", otp, created_at, updated_at"#,
            otp,
            self.phone_number
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub id: Option<Uuid>,
    pub phone_number: Option<String>,
    pub name: Option<String>,
    pub nik: Option<String>,
    pub role: Option<Role>,
    pub status: Option<UserStatus>,
    pub otp: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl UpdateUser {
    pub async fn update(self, phone_number: String, pool: &Pool<Postgres>) -> Result<User> {
        let user = sqlx::query_as!(
            User, 
            r#"
                update "user" 
                    set phone_number = coalesce($1, "user".phone_number), 
                        name = coalesce($2, "user".name), 
                        nik = coalesce($3, "user".nik), 
                        otp = coalesce($4, "user".otp), 
                        created_at = coalesce($5, "user".created_at), 
                        updated_at = coalesce($6, "user".updated_at),
                        status = coalesce($7, "user".status),
                        role = coalesce($8, "user".role)
                    where phone_number = $9
                    returning id, phone_number, name, nik, role as "role!: Role", status as "status!: UserStatus", otp, created_at, updated_at
            "#,  
            self.phone_number,
            self.name,
            self.nik,
            self.otp,
            self.created_at,
            self.updated_at,
            self.status.unwrap_or(UserStatus::Default) as UserStatus,
            self.role.unwrap_or(Role::Default) as Role,
            phone_number
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }
}
