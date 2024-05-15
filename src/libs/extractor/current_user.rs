use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, Validation};
use serde::{Deserialize, Serialize};

use crate::{error::aggregate::Error, jwt::config::KEYS};

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}

#[async_trait]
#[cfg_attr(
    nightly_error_messages,
    diagnostic::on_unimplemented(
        note = "Function argument is not a valid axum extractor. \nSee `https://docs.rs/axum/0.7/axum/extract/index.html` for details",
    )
)]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                Error::Unauthorize("This API cannot be used without an API Key".to_string())
            })?;

        let token_data: jsonwebtoken::TokenData<CurrentUser> =
            decode::<CurrentUser>(bearer.token(), &KEYS.decoding, &Validation::default())
                .map_err(|_| Error::Unauthorize("API Key is not valid".to_string()))?;

        Ok(token_data.claims)
    }
}
