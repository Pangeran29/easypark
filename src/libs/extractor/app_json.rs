use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(FromRequest)]
#[from_request(via(axum::Json))]
pub struct AppSuccess<T>(pub T);

impl<T> IntoResponse for AppSuccess<T>
where
    axum::Json<SuccessResponse<T>>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(SuccessResponse {
            success: true,
            data: self.0,
        })
        .into_response()
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json))]
pub struct AppFailed<T>(pub T);

impl<T> IntoResponse for AppFailed<T>
where
    axum::Json<ErrorResponse<T>>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(ErrorResponse {
            success: false,
            message: self.0,
        }).into_response()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse<T> {
    pub success: bool,
    pub message: T,
}

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}
