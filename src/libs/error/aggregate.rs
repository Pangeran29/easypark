use crate::extractor::app_json::AppFailed;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::debug;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BadRequest(String),
    NotFoundRejection(String),
    Unauthorize(String),
    InternalServerError(String),
    JsonRejection(JsonRejection),
    Sqlx(sqlx::Error),
    Reqwest(reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Error::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Error::NotFoundRejection(message) => (StatusCode::NOT_FOUND, message),
            Error::Unauthorize(message) => (StatusCode::UNAUTHORIZED, message),
            Error::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            Error::Sqlx(err) => {
                let (status, message) = match err {
                    sqlx::Error::Database(dbx) => (StatusCode::CONFLICT, dbx.message().to_owned()),
                    sqlx::Error::RowNotFound => {
                        (StatusCode::CONFLICT, "Data not found".to_string())
                    }
                    _ => {
                        debug!("Fail to do DB operation: {}", err.to_string());
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Something went wrong".to_owned(),
                        )
                    }
                };
                (status, message)
            }
            Error::Reqwest(err) => {
                debug!("Fail to send whatsapp: {}", err.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Fail to send whatsapp".to_owned(),
                )
            }
        };
        (status, AppFailed(message)).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}
