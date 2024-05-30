use crate::{
    app::user::{Role, User, UserStatus},
    error::aggregate::{Error, Result},
    extractor::{app_body::Body, app_json::AppSuccess, current_user::CurrentUser},
    jwt::config::KEYS,
    middleware::base::print_request_body,
};
use axum::{extract::State, middleware, routing::post, Router};
use chrono::{DateTime, Datelike, Duration, Utc};
use jsonwebtoken::{encode, Header};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

pub fn build(pool: Pool<Postgres>) -> Router {
    let router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/activate-phone-number", post(activate_phone_number))
        .layer(middleware::from_fn(print_request_body))
        .with_state(pool);

    Router::new().nest("/auth", router)
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterPayload {
    phone_number: String,
    name: String,
    nik: String,
    role: Role,
}

impl RegisterPayload {
    fn into_user(self) -> User {
        User {
            id: Uuid::new_v4(),
            phone_number: self.phone_number,
            name: self.name,
            nik: self.nik,
            role: self.role,
            status: UserStatus::NotActive,
            otp: None,
            created_at: None,
            updated_at: None,
            parking_lot_id: None,
            owner_id: None
        }
    }
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    phone: String,
    otp: i32,
}

#[derive(Debug, Serialize)]
struct AuthMeta {
    expired_in: DateTime<Utc>,
    token_type: String,
}

#[derive(Debug, Serialize)]
struct Login {
    token: String,
    user: User,
    token_meta: AuthMeta,
}

async fn register(
    State(pool): State<PgPool>,
    Body(payload): Body<RegisterPayload>,
) -> Result<AppSuccess<User>> {
    let user = payload.into_user();
    let user = user.save(&pool).await?;
    Ok(AppSuccess(user))
}

async fn login(
    State(pool): State<PgPool>,
    Body(payload): Body<LoginPayload>,
) -> Result<AppSuccess<Login>> {
    let LoginPayload { phone, otp } = payload;
    let user = User::find_one(phone, &pool).await?;

    if user.status == UserStatus::NotActive {
        return Err(Error::BadRequest("Account not active".to_string()));
    }

    let user_otp = user
        .otp
        .ok_or_else(|| Error::BadRequest("OTP not valid".to_string()))?;

    if user_otp != otp {
        return Err(Error::BadRequest("OTP not valid".to_string()));
    }

    let user = user.invalidate_otp(&pool).await?;

    let now = Utc::now();
    let exp = now
        .with_year(now.year() + 1)
        .unwrap_or_else(|| now + Duration::days(365));

    let current_user = CurrentUser {
        sub: user.phone_number.to_owned(),
        company: "Backend Parking".to_owned(),
        exp: exp.timestamp() as usize,
    };

    let token = encode(&Header::default(), &current_user, &KEYS.encoding)
        .map_err(|_| Error::InternalServerError("Fail to login".to_string()))?;

    Ok(AppSuccess(Login {
        token,
        user,
        token_meta: AuthMeta {
            expired_in: exp,
            token_type: "Bearer".to_string(),
        },
    }))
}

async fn activate_phone_number(
    State(pool): State<PgPool>,
    Body(payload): Body<LoginPayload>,
) -> Result<AppSuccess<Login>> {
    let LoginPayload { phone, otp } = payload;
    let user = User::find_one(phone, &pool).await?;

    let user_otp = user
        .otp
        .ok_or_else(|| Error::BadRequest("OTP not valid".to_string()))?;

    if user_otp != otp {
        return Err(Error::BadRequest("OTP not valid".to_string()));
    }

    let user = user.update_status(UserStatus::Active, &pool).await?;
    let user = user.invalidate_otp(&pool).await?;

    let now = Utc::now();
    let exp = now
        .with_year(now.year() + 1)
        .unwrap_or_else(|| now + Duration::days(365));

    let current_user = CurrentUser {
        sub: user.phone_number.to_owned(),
        company: "Backend Parking".to_owned(),
        exp: exp.timestamp() as usize,
    };

    let token = encode(&Header::default(), &current_user, &KEYS.encoding)
        .map_err(|_| Error::InternalServerError("Fail to login".to_string()))?;

    Ok(AppSuccess(Login {
        token,
        user,
        token_meta: AuthMeta {
            expired_in: exp,
            token_type: "Bearer".to_string(),
        },
    }))
}
