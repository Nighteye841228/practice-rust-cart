use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::errors::AppError::{ClientError, ServerError};

pub enum AppError {
    ClientError(StatusCode, BusinessCode, String),
    ServerError(anyhow::Error),
}

pub enum BusinessCode {
    PasswordTooShort,
    // Duplicate,
    EmptyField,
    WrongPassword,
    NoAuth,
    TokenExpired,
    NotFoundUser,
}

impl BusinessCode {
    fn as_str(&self) -> String {
        match self {
            // BusinessCode::Duplicate => String::from("DUPLICATE"),
            BusinessCode::PasswordTooShort => String::from("PASSWORD_TOO_SHORT"),
            BusinessCode::EmptyField => String::from("EMPTY_FIELD"),
            BusinessCode::WrongPassword => String::from("WRONG_PASSWORD"),
            BusinessCode::NoAuth => String::from("NO_AUTH"),
            BusinessCode::TokenExpired => String::from("TOKEN_EXPIRED"),
            BusinessCode::NotFoundUser => String::from("NOT_FOUND_USER"),
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    code: String,
    msg: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ClientError(status, code, msg) => (
                status,
                Json(ErrorResponse {
                    code: code.as_str(), // 直接轉字串，不用再 match 一次
                    msg,
                }),
            )
                .into_response(),
            ServerError(e) => {
                println!("Server Error: {}", e.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        code: String::from("SERVER_ERROR"),
                        msg: String::from("server busy, please wait..."),
                    }),
                )
                    .into_response()
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::ServerError(err.into())
    }
}

// 1. 定義一個特徵
pub trait OptionExt<T> {
    fn or_bad_request(self, msg: &str) -> Result<T, AppError>;
    fn or_not_found(self, msg: &str) -> Result<T, AppError>;
}

// 2. 實作在標準庫的 Option 上
impl<T> OptionExt<T> for Option<T> {
    fn or_bad_request(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| {
            AppError::ClientError(
                StatusCode::BAD_REQUEST,
                BusinessCode::EmptyField,
                msg.to_string(),
            )
        })
    }

    fn or_not_found(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| {
            AppError::ClientError(
                StatusCode::NOT_FOUND,
                BusinessCode::NotFoundUser,
                msg.to_string(),
            )
        })
    }
}
