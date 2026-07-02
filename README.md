# Rust Cart API

> 使用 Rust、Axum、PostgreSQL 與 SQLx 開發中的電商後端練習專案。

Rust Cart API 是一個以「逐步完成可維護電商系統」為目標的後端專案。  
目前聚焦於帳號驗證、Token 管理、商品建立與圖片上傳；後續將持續完成購物車、訂單、金流、高併發處理、測試與部署。

> **目前狀態：WIP（持續開發中）**  
> 此專案尚未適合直接部署到正式環境。

## 專案目標

透過此專案練習並展示：

- 使用 Rust 與 Axum 建立 RESTful API
- PostgreSQL 資料建模與 SQLx 資料存取
- JWT、Cookie 與 Refresh Token 驗證流程
- 密碼雜湊、重設密碼與權限設計
- 商品與圖片上傳流程
- 訂單、金流、庫存與狀態機設計
- Redis、RabbitMQ、限流、測試、Docker 與 CI/CD 的後端工程實務

## 技術棧

| 類別              | 技術                                |
| ----------------- | ----------------------------------- |
| 程式語言          | Rust 2024 Edition                   |
| Web Framework     | Axum                                |
| 非同步 Runtime    | Tokio                               |
| 資料庫            | PostgreSQL                          |
| ORM / Query Layer | SQLx                                |
| 驗證              | JWT、HttpOnly Cookie、Refresh Token |
| 密碼雜湊          | Argon2id                            |
| Email             | Resend、Askama                      |
| 檔案上傳          | Axum Multipart                      |
| 靜態檔案          | tower-http `ServeDir`               |
| ID                | UUID v7                             |

## 目前已完成

### 帳號與驗證

- [x] 使用者註冊
- [x] 使用 Argon2id 雜湊密碼
- [x] 使用 Email 與密碼登入
- [x] JWT Access Token
- [x] 將 Access Token 寫入 HttpOnly Cookie
- [x] Refresh Token 產生與資料庫保存
- [x] Refresh Token 換發新的 Access Token
- [x] 自訂 `Claims` Extractor 保護路由
- [x] 登出流程與 Cookie 清除
- [x] 寄送重設密碼 Email
- [x] 重設密碼 Token 驗證
- [x] 以 Transaction 完成「更新密碼 + 移除重設 Token」
- [x] 使用者帳號刪除

### 商品管理

- [x] 商品資料表與 `draft` / `published` / `archived` 狀態
- [x] 建立商品 API
- [x] 商品規格、售價、庫存、配送方式與內部備註欄位
- [x] 暫存商品圖片上傳
- [x] JPEG / PNG 圖片格式驗證
- [x] 商品建立時，將圖片從暫存目錄移至正式商品圖片目錄
- [ ] 管理員權限保護
- [ ] 商品查詢、搜尋與篩選
- [ ] 商品修改與下架
- [ ] 對外商品資料模型
- [ ] 我的最愛

### 資料庫與專案結構

- [x] `users` 資料表
- [x] `refresh_tokens` 資料表
- [x] `password_reset_tokens` 資料表
- [x] `products` 資料表
- [x] SQL migration 檔案
- [ ] 啟動服務時自動執行 migration
- [ ] 測試資料庫與整合測試
- [ ] Docker 開發環境

## 目前 API

| Method | Endpoint                     | 說明                               | 驗證狀態             |
| ------ | ---------------------------- | ---------------------------------- | -------------------- |
| `GET`  | `/`                          | 測試 PostgreSQL 連線               | 不需要               |
| `POST` | `/register`                  | 註冊使用者                         | 不需要               |
| `POST` | `/login`                     | 登入並設定 Access / Refresh Cookie | 不需要               |
| `POST` | `/refresh`                   | 以 Refresh Token 換發 Access Token | Refresh Cookie       |
| `POST` | `/logout`                    | 登出並清除 Cookie                  | JWT + Refresh Cookie |
| `POST` | `/delete`                    | 刪除目前使用者帳號                 | JWT                  |
| `GET`  | `/test`                      | 測試 JWT 保護路由                  | JWT                  |
| `POST` | `/send-reset-password-email` | 寄送重設密碼信                     | 不需要               |
| `POST` | `/reset-password`            | 以重設 Token 更新密碼              | 不需要               |
| `POST` | `/product/create`            | 建立商品                           | 目前尚未限制管理員   |
| `POST` | `/product/image-upload`      | 上傳商品暫存圖片                   | 目前尚未限制管理員   |

## 驗證流程

```text
註冊
  ↓
使用 Argon2id 雜湊密碼並寫入 PostgreSQL
  ↓
登入
  ↓
產生短效 Access Token 與 Refresh Token
  ↓
Access Token 寫入 HttpOnly Cookie
Refresh Token 寫入 HttpOnly Cookie，並將雜湊值保存至資料庫
  ↓
受保護路由由 Claims Extractor 驗證 Access Token
  ↓
Access Token 過期時，由 /refresh 換發新 Token
```

## 商品圖片流程

```text
上傳圖片
  ↓
驗證檔案類型為 JPEG 或 PNG
  ↓
儲存至 ./temp 暫存目錄
  ↓
回傳暫存圖片檔名
  ↓
建立商品時提供圖片檔名
  ↓
將檔案移至 ./assets/product
  ↓
寫入商品資料庫紀錄
```

## 專案結構

```text
.
├── assets/
│   └── product/                 # 商品正式圖片
├── migrations/                  # PostgreSQL migration SQL
├── src/
│   ├── extractors/
│   │   └── jwt.rs               # JWT Claims Extractor
│   ├── handlers/
│   │   └── products.rs          # 商品 HTTP handlers
│   ├── models/
│   │   └── products.rs          # 圖片上傳與檔案處理
│   ├── repositories/
│   │   └── products.rs          # 商品資料存取
│   ├── errors.rs                # 應用程式錯誤格式
│   ├── extractors.rs            # Extractor module
│   ├── handlers.rs              # 帳號與驗證 handlers
│   ├── main.rs                  # Router 與應用程式進入點
│   ├── models.rs                # Model module
│   ├── repositories.rs          # Repository module
│   └── user_repo.rs             # 使用者 request / response DTO
├── templates/
│   └── resend_email.html        # 重設密碼 Email 模板
├── .env.example
├── Cargo.toml
└── LICENSE
```

## 本機啟動

### 前置需求

- Rust toolchain
- PostgreSQL
- SQLx CLI
- Resend API Key（僅寄送重設密碼 Email 時需要）

### 1. Clone 專案

```bash
git clone https://github.com/Nighteye841228/practice-rust-cart.git
cd practice-rust-cart
```

### 2. 建立環境變數檔

```bash
cp .env.example .env
```

編輯 `.env`：

```env
DATABASE_URL=postgres://<user>:<password>@localhost/<database_name>
JWT_SECRET=<replace-with-a-long-random-secret>
RESEND_API=<your-resend-api-key>
```

### 3. 建立資料庫並執行 migration

```bash
createdb rust_cart

cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

### 4. 啟動服務

```bash
cargo run
```

開發伺服器預設監聽：

```text
http://127.0.0.1:3000
```

## 開發規劃

### Phase 1：核心業務與權限模型

- [x] JWT 基礎身分驗證
- [ ] `AdminClaims` Extractor：限制後台 API 只能由管理員使用
- [ ] 管理後台商品搜尋、讀取、修改與下架
- [ ] 前台商品搜尋、分類過濾與公開資料模型
- [ ] 我的最愛（Wishlist）
- [ ] 動態定價：VIP、折扣碼與滿額優惠
- [ ] 購物車：讀取、更新數量、刪除

### Phase 2：訂單、金流與狀態機

- [ ] 建立訂單商品快照，避免商品價格變動影響歷史訂單
- [ ] 訂單狀態機：`Pending`、`Paid`、`Shipped`、`Completed`、`Cancelled`
- [ ] 多收件地址管理
- [ ] 串接 Stripe 或綠界科技
- [ ] 驗證金流 Webhook，安全更新付款狀態
- [ ] 訂單留言板與訂單歷史分頁
- [ ] 以背景工作寄送訂單通知 Email

### Phase 3：高併發與安全

- [ ] Redis 購物車
- [ ] Redis Lua 腳本預扣庫存，避免超賣
- [ ] RabbitMQ 訂單事件佇列，處理尖峰流量
- [ ] 使用 `tower-governor` 做登入與敏感 API 限流
- [ ] 使用 `tower-http` 設定 CORS
- [ ] 使用 `tracing` 建立結構化日誌與 Request ID
- [ ] 資料庫索引與查詢效能優化

### Phase 4：重構、文件與測試

- [ ] 視專案規模拆分 Cargo Workspace
- [ ] 使用 `utoipa` 產生 OpenAPI / Swagger UI
- [ ] 動態定價、JWT、狀態機等核心邏輯單元測試
- [ ] 註冊 → 登入 → 建立商品等 API 整合測試
- [ ] Test database 與測試資料清理流程

### Phase 5：部署與維運

- [ ] Docker multi-stage build
- [ ] Docker Compose 本機開發環境
- [ ] `GET /health` 健康檢查
- [ ] Graceful Shutdown
- [ ] Caddy 反向代理與自動 HTTPS
- [ ] GitHub Actions：`cargo fmt`、`cargo clippy`、`cargo test`
- [ ] 自動建置 Docker image 與部署

## 下一個里程碑

下一步將優先完成：

1. `AdminClaims` 與角色欄位，先保護商品管理 API。
2. 商品查詢、修改與下架，完成基本商品管理流程。
3. 購物車資料模型與 CRUD。
4. 訂單建立與商品快照，讓電商核心流程可以開始串接。
5. 導入測試與 OpenAPI 文件，讓 API 可被驗證與使用。

## 安全與正式環境注意事項

目前專案仍為學習用途。正式部署前至少需要完成：

- 啟用 HTTPS，並將 Cookie 設為 `Secure`
- 保護所有後台商品 API
- 補上輸入驗證與權限測試
- 移除開發環境使用的 Token 回傳或除錯訊息
- 改善 Refresh Token 失效與輪替策略
- 增加限流、CORS、結構化日誌與監控
- 建立整合測試與部署前檢查
