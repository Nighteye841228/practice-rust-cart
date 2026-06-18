use axum::{body::Bytes, extract::Multipart, http::StatusCode};
use tokio::fs::{self, create_dir_all};
use uuid::Uuid;

use crate::{
    errors::{AppError, BusinessCode, OptionExt},
    repositories::products::ProductTempImageResponse,
};

pub async fn write_image_to_temp_dir(
    mut image_content: Multipart,
) -> Result<ProductTempImageResponse, AppError> {
    // 1. 在迴圈外準備容器，用來收集表單資料
    let mut alt_text: Option<String> = None;
    let mut image_buf: Option<Bytes> = None;
    let mut file_extension: String = String::new();

    // 2. 開始逐一解析表單欄位
    while let Some(field) = image_content.next_field().await? {
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
                image_buf = Some(field.bytes().await?);
                // 在這裡才去檢查 Content-Type 是否為合法圖片
                file_extension =
                    tell_png_jpg(image_buf.as_ref().or_not_found("no images")?).await?;
            }
            _ => {
                // 忽略其他不認識的欄位
            }
        }
    }

    let data = image_buf.or_not_found("no images")?;
    let dest = save_temp_local(data, &file_extension).await?;
    println!("final dest is: {}", dest);

    Ok(ProductTempImageResponse {
        url: dest,
        alt: alt_text.or_not_found("")?,
    })
}

async fn tell_png_jpg(buffer: &Bytes) -> Result<String, AppError> {
    let kind = infer::get(buffer).or_not_found("no buffer.")?;
    match kind.mime_type() {
        "image/jpeg" => Ok(String::from("jpg")),
        "image/png" => Ok(String::from("png")),
        _ => Err(AppError::ClientError(
            StatusCode::BAD_REQUEST,
            BusinessCode::EmptyField,
            "wrong type".to_string(),
        )),
    }
}

/// # 記得要實作實際上把檔案轉入asset的過程
async fn save_temp_local(buffer: Bytes, extension: &str) -> Result<String, AppError> {
    // let image_dir_id = Utc::now().format("%Y/%m/%d").to_string();
    let image_id = Uuid::now_v7().to_string();
    create_dir_all("./temp").await?;
    let image_dest = format!("./temp/{image_id}.{extension}");
    fs::write(&image_dest, buffer).await?;

    Ok(format!("{image_id}.{extension}"))
}
