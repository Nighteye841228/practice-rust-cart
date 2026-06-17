use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow, types::Json};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(sqlx::Type, Debug, Deserialize)]
#[sqlx(rename_all = "lowercase", type_name = "product_status")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Draft,
    Published,
    Archived,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    weight: u32,
    ingredient: Vec<String>,
    exp: String,
    saving_method: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Images {
    description: String,
    url: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct ProductsCreate {
    pub name: String,
    pub description: String,
    pub specs: Spec,
    pub images: Vec<Images>,
    pub status: Status,
    pub original_price: i32,
    pub selling_price: i32,
    pub stock_available: i32,
    pub shipping_methods: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct ProductsCreateResponse {
    pub id: Uuid,
}

pub async fn product_insert(
    payload: ProductsCreate,
    pool: &PgPool,
) -> Result<ProductsCreateResponse, AppError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO products
        (name, description, specs, images, status, original_price, selling_price, stock_available, shipping_methods, notes)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        payload.name,
        payload.description,
        Json(payload.specs) as Json<Spec>,
        Json(payload.images) as Json<Vec<Images>>,
        payload.status as Status,
        payload.original_price,
        payload.selling_price,
        payload.stock_available,
        Json(payload.shipping_methods) as Json<Vec<String>>,
        payload.notes
    ).fetch_one(pool).await?;

    Ok(ProductsCreateResponse { id: row.id })
}
