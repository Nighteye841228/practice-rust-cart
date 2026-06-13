use std::env;

use axum::{extract::FromRequestParts, http::StatusCode};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, BusinessCode::NoAuth};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // sub (Subject): 通行證的主人，通常放 User ID
    pub sub: String,
    // exp (Expiration time): 通行證何時失效，必須是 Unix Timestamp (秒數)
    pub exp: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await?;
        let Some(token) = jar.get("access-token") else {
            return Err(AppError::ClientError(
                StatusCode::BAD_REQUEST,
                NoAuth,
                "no auth provided".to_owned(),
            ));
        };

        let claim = decode::<Claims>(
            token.value(),
            &DecodingKey::from_secret(env::var("JWT_SECRET")?.as_ref()),
            &Validation::default(),
        )?;

        Ok(claim.claims)
    }
}
