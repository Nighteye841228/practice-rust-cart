use crate::{
    errors::{AppError, BusinessCode, OptionExt},
    repositories::products::{ProductsCreate, ProductsCreateResponse, product_insert},
};
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
};
use sqlx::PgPool;

pub async fn create(
    State(pool): State<PgPool>,
    Json(payload): Json<ProductsCreate>,
) -> Result<Json<ProductsCreateResponse>, AppError> {
    let res = product_insert(payload, &pool).await?;

    Ok(Json(res))
}

pub async fn save_images(mut multipart: Multipart) -> Result<String, AppError> {
    // 1. 在迴圈外準備容器，用來收集表單資料
    let mut alt_text: Option<String>;
    let mut image_data: Option<axum::body::Bytes> = None;
    let mut file_extension: String;

    // 2. 開始逐一解析表單欄位
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            // 處理 alt 文字欄位
            "alt" => {
                let text = field.text().await?;
                alt_text = Some(text);
                println!("收到 alt 說明: {:?}", alt_text);
            }
            // 處理圖片檔案欄位 (假設你在 HTTP 請求裡把這欄叫做 upload_file)
            "upload_file" => {
                let content_type = field.content_type().unwrap_or("").to_string();

                let image_buf = field.bytes().await?;
                // 在這裡才去檢查 Content-Type 是否為合法圖片
                if !is_png_jpg(&image_buf).await? {
                    return Err(AppError::ClientError(
                        StatusCode::BAD_REQUEST,
                        BusinessCode::EmptyField, // 建議可以新增一個 InvalidFormat 錯誤碼
                        "Only image files are allowed.".to_string(),
                    ));
                }

                // 紀錄副檔名與二進位資料
                file_extension = content_type.replace("image/", "");
                image_data = Some(image_buf);
                if let Some(data) = &image_data {
                    println!(
                        "收到圖片檔案，大小: {} bytes, 副檔名: {}",
                        data.len(),
                        file_extension
                    );
                }
            }
            _ => {
                // 忽略其他不認識的欄位
            }
        }
    }

    // 3. 迴圈結束後，確認必須的資料都有收到
    let Some(data) = image_data else {
        return Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::EmptyField,
            "No image file uploaded.".to_string(),
        ));
    };

    // --- 這裡可以開始實作將 data 寫入硬碟的邏輯 ---
    // 你可以同時把 alt_text 和回傳的圖片 URL 組合起來還給前端

    Ok("OK".to_owned())
}

async fn is_png_jpg(buffer: &Bytes) -> Result<bool, AppError> {
    let kind = infer::get(buffer).or_not_found("no buffer.")?;

    Ok(kind.mime_type() == "image/jpeg" || kind.mime_type() == "image/png")
}
