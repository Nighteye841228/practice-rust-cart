use crate::{
    errors::{AppError, BusinessCode, OptionExt},
    extractors::jwt::AdminClaims,
    models::products::write_image_to_temp_dir,
    repositories::products::{
        ProductTempImageResponse, ProductsCreate, ProductsCreateResponse, product_insert,
    },
};
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::PgPool;
use tokio::fs::{self, create_dir_all};
use uuid::Uuid;

pub async fn create(
    State(pool): State<PgPool>,
    _: AdminClaims,
    Json(payload): Json<ProductsCreate>,
) -> Result<Json<ProductsCreateResponse>, AppError> {
    let res = product_insert(payload, &pool).await?;

    Ok(Json(res))
}

pub async fn upload_temp_image(
    _: AdminClaims,
    multipart: Multipart,
) -> Result<Json<ProductTempImageResponse>, AppError> {
    let res = write_image_to_temp_dir(multipart).await?;

    Ok(Json(res))
}
