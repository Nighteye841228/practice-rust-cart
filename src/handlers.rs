pub mod products;

use crate::extractors::jwt::Role;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use askama::Template;
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Duration, Utc};
use cookie::time::{self, Duration as CookieDuration};
use jsonwebtoken::{EncodingKey, Header, encode};
use password_worker::{Argon2idConfig, PasswordWorker};
use resend_rs::{Resend, types::CreateEmailBaseOptions};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use crate::{
    errors::{
        AppError,
        BusinessCode::{self, NoAuth},
    },
    extractors::jwt::Claims,
    user_repo::{
        UserDeleteResponse, UserLogin, UserRegister, UserRegisterResponse, UserResetPassword,
        UserResetPasswordEmail, UserResetPasswordEmailResponse, UserResetPasswordResponse,
    },
};

#[derive(Template)]
#[template(path = "./resend_email.html")]
struct ResetPasswordTemplate {
    url: String,
}

#[axum::debug_handler]
pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserRegister>,
) -> Result<Json<UserRegisterResponse>, AppError> {
    if payload.password.len() < 8 {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            crate::errors::BusinessCode::PasswordTooShort,
            String::from("password should exceed 8 characters"),
        ));
    }

    let hash_code = argon_hash(&payload.password).await?;

    let db_response = sqlx::query!(
        "INSERT INTO users (account, email, password, shipping_address, name, recipient_name, phone) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        payload.account,
        payload.email,
        hash_code,
        payload.shipping_address,
        payload.name,
        payload.recipient_name,
        payload.phone
    ).fetch_one(&pool).await?;

    println!("id is: {}", db_response.id);
    Ok(Json(UserRegisterResponse {
        id: db_response.id.into(),
    }))
}

pub async fn login(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<UserLogin>,
) -> Result<CookieJar, AppError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::EmptyField,
            String::from("Not insert email or password"),
        ));
    }

    let user = sqlx::query!(
        r#"SELECT id, password, role as "role: Role"
        FROM users WHERE email=$1"#,
        payload.email,
    )
    .fetch_optional(&pool)
    .await?;

    let hasher = PasswordWorker::new_argon2id(4)?;
    let Some(row) = user else {
        let dummy_hash = "$2b$10$92IXUNpkjO0rOQ5byMi.Ye4oKoEa3Ro9llC/.og/at2.uheWG/igi";
        // 3. 執行完整的驗證，讓它消耗與密碼錯誤時完全相同的 CPU 時間
        let _ = PasswordWorker::verify(&hasher, &payload.password, dummy_hash).await;

        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::WrongPassword,
            "Wrong email or username.".to_string(),
        ));
    };

    if !PasswordWorker::verify(&hasher, &payload.password, &row.password).await? {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::WrongPassword,
            "Wrong email or username.".to_string(),
        ));
    }

    let expiration = Utc::now() + Duration::seconds(15);
    let secret = env::var("JWT_SECRET")?;
    let access_token = encode(
        &Header::default(),
        &Claims {
            sub: row.id.to_string(),
            exp: expiration.timestamp().try_into()?,
            role: row.role,
        },
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    let refresh_token = Uuid::now_v7().to_string();
    let hashed_refresh_token = sha256_hash(refresh_token.as_str()).await?;
    let expire_time = Utc::now() + Duration::days(7);

    sqlx::query!(
        "INSERT INTO refresh_tokens (token, user_id, expires_at) VALUES ($1, $2, $3) RETURNING id",
        hashed_refresh_token,
        row.id,
        expire_time
    )
    .fetch_one(&pool)
    .await?;

    let access_cookie = Cookie::build(("access-token", access_token))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::seconds(10));

    let refresh_cookie = Cookie::build(("refresh-token", refresh_token))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::days(10));

    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok(jar)
}

pub async fn refresh(State(pool): State<PgPool>, jar: CookieJar) -> Result<CookieJar, AppError> {
    let Some(expired_refresh_token) = jar.get("refresh-token") else {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::NoAuth,
            "refresh token fetch failed.".to_string(),
        ));
    };
    let recv_refresh_hash_code = sha256_hash(expired_refresh_token.value()).await?;
    let refresh_row = sqlx::query!(
        r#"
        SELECT r.user_id, r.token, r.expires_at, u.role as "role: Role"
        FROM refresh_tokens as r
        INNER JOIN users as u
        ON r.user_id=u.id
        WHERE token=$1
        "#,
        recv_refresh_hash_code
    )
    .fetch_optional(&pool)
    .await?;

    let Some(refresh_row) = refresh_row else {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::NoAuth,
            "refresh token fetch failed.".to_string(),
        ));
    };

    if Utc::now() > refresh_row.expires_at.to_utc() {
        return Err(AppError::ClientError(
            StatusCode::UNAUTHORIZED,
            BusinessCode::TokenExpired,
            "Need to login".to_string(),
        ));
    }

    let expiration = Utc::now() + Duration::minutes(5);
    let secret = env::var("JWT_SECRET")?;
    let access_token = encode(
        &Header::default(),
        &Claims {
            sub: refresh_row.user_id.to_string(),
            exp: expiration.timestamp().try_into()?,
            role: refresh_row.role,
        },
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    let access_cookie = Cookie::build(("access-token", access_token))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(10));

    let jar = jar.add(access_cookie);

    Ok(jar)
}

pub async fn logout(
    State(pool): State<PgPool>,
    user: Claims,
    jar: CookieJar,
) -> Result<CookieJar, AppError> {
    let Some(refresh_token) = jar.get("refresh-token") else {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            NoAuth,
            "no auth provided".to_owned(),
        ));
    };

    let id = Uuid::parse_str(&user.sub)?;
    let _ = sqlx::query!(
        "DELETE FROM refresh_tokens where token=$1 AND user_id=$2",
        refresh_token.value(),
        id
    )
    .execute(&pool)
    .await?;

    let access_cookie = Cookie::build(("access-token", ""))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::milliseconds(0));

    let refresh_cookie = Cookie::build(("refresh-token", ""))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::milliseconds(0));

    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok(jar)
}

pub async fn delete(
    State(pool): State<PgPool>,
    user: Claims,
    jar: CookieJar,
) -> Result<(CookieJar, Json<UserDeleteResponse>), AppError> {
    let user_id = Uuid::parse_str(&user.sub)?;
    sqlx::query!("DELETE FROM users WHERE id=$1", user_id)
        .execute(&pool)
        .await?;

    let access_cookie = Cookie::build(("access-token", ""))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::milliseconds(0));

    let jar = jar.add(access_cookie);

    Ok((
        jar,
        Json(UserDeleteResponse {
            msg: "delete user success".to_string(),
        }),
    ))
}

pub async fn send_reset_password_email(
    State(pool): State<PgPool>,
    Json(payload): Json<UserResetPasswordEmail>,
) -> Result<Json<UserResetPasswordEmailResponse>, AppError> {
    let Some(row) = sqlx::query!("SELECT id, email FROM users WHERE email=$1", &payload.email)
        .fetch_optional(&pool)
        .await?
    else {
        return Ok(Json(UserResetPasswordEmailResponse {
            msg: format!("Reset email is sent"),
        }));
    };

    let reset_token = Uuid::now_v7().to_string();
    let token = sha256_hash(&reset_token).await?;
    let expire_time = Utc::now() + Duration::minutes(10);
    sqlx::query!(
        "INSERT INTO password_reset_tokens (token, user_id, expires_at) VALUES ($1, $2, $3)",
        &token,
        row.id,
        expire_time
    )
    .execute(&pool)
    .await?;

    //send email
    let resend_api_key = env::var("RESEND_API")?;
    let resend = Resend::new(&resend_api_key);

    let from = "onboarding@resend.dev";
    let to = ["victor70412@gmail.com"];
    let subject = "重置你的密碼";

    let email_render = ResetPasswordTemplate {
        url: format!("http://localhost:3000/reset-password?token={}", reset_token),
    }; // instantiate your struct
    let html = email_render.render()?;

    let email = CreateEmailBaseOptions::new(from, to, subject).with_html(&html);

    let _email = resend.emails.send(email).await?;
    println!("{:?}", _email);

    Ok(Json(UserResetPasswordEmailResponse {
        msg: format!("Reset email is sent. temp token is: {}", reset_token),
    }))
}

pub async fn reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<UserResetPassword>,
) -> Result<Json<UserResetPasswordResponse>, AppError> {
    let token = sha256_hash(&payload.token).await?;
    let query = sqlx::query!(
        "SELECT user_id FROM password_reset_tokens WHERE token=$1 AND expires_at > $2",
        token,
        &Utc::now()
    )
    .fetch_optional(&pool)
    .await?;

    let Some(row) = query else {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::NotFoundUser,
            "cant reset password".to_string(),
        ));
    };

    let hash_code = argon_hash(&payload.password).await?;

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE users SET password=$1 where id=$2",
        hash_code,
        &row.user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "DELETE FROM password_reset_tokens WHERE user_id=$1",
        &row.user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(UserResetPasswordResponse {
        msg: "reset password successfully.".to_string(),
    }))
}

async fn argon_hash(input: &str) -> Result<String, AppError> {
    let hasher = PasswordWorker::new_argon2id(4)?;
    let salt = SaltString::generate(&mut OsRng).to_string().into_bytes();
    Ok(hasher
        .hash(
            input,
            Argon2idConfig {
                salt,
                time_cost: 2,
                mem_cost: 65536,
                hash_length: 32,
            },
        )
        .await?)
}

async fn sha256_hash(input: &str) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());

    Ok(hex::encode(hasher.finalize()))
}

pub async fn test(State(_pool): State<PgPool>, _user: Claims) -> Result<(), AppError> {
    println!("test success");
    Ok(())
}
