pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod user_repo;
use std::time::Duration;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::services::ServeDir;

use crate::handlers::{
    delete, login, logout,
    products::{create, upload_temp_image},
    refresh, register, reset_password, send_reset_password_email, test,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("cannot connect to database");

    let app = Router::new()
        .nest_service("/product_edit_images", ServeDir::new("temp"))
        .route("/", get(using_connection_pool_extractor))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/delete", post(delete))
        .route("/test", get(test))
        .route(
            "/send-reset-password-email",
            post(send_reset_password_email),
        )
        .route("/reset-password", post(reset_password))
        .route("/product/create", post(create))
        .route("/product/image-upload", post(upload_temp_image))
        .with_state(pool);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

// async fn handler() -> Html<&'static str> {
//     Html("<h1>Hello, World!</h1>")
// }

async fn using_connection_pool_extractor(
    State(pool): State<PgPool>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
