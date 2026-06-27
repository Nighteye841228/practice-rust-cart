use std::{
    env,
    fmt::{self, Display},
};

use axum::{extract::FromRequestParts, http::StatusCode};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::errors::{
    AppError,
    BusinessCode::{self, NoAuth},
};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum Role {
    Admin,
    Client,
}

impl Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_representation = match self {
            Role::Admin => "Admin",
            Role::Client => "Client",
        };
        write!(f, "{}", string_representation)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // sub (Subject): 通行證的主人，通常放 User ID
    pub sub: String,
    // exp (Expiration time): 通行證何時失效，必須是 Unix Timestamp (秒數)
    pub exp: usize,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims(pub Claims);

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

impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;
        match claims.role {
            Role::Admin => Ok(AdminClaims(claims)),
            _ => {
                return Err(AppError::ClientError(
                    StatusCode::BAD_REQUEST,
                    BusinessCode::NoAuth,
                    format!("Admin authorize failed"),
                ));
            }
        }
    }
}
