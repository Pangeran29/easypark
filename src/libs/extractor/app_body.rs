use crate::libs::error::aggregate::Error;
use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
};

#[derive(Clone, FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Body<T>(pub T);

impl<T> IntoResponse for Body<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
