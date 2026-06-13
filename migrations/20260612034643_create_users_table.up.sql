-- Add up migration script here
-- Add migration script here
CREATE TABLE users (
    -- 主鍵：使用 PostgreSQL 13+ 內建的 UUID 產生函數
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    
    -- 帳號：不可為空、不可重複
    account VARCHAR(50) UNIQUE NOT NULL,
    
    -- 密碼：不可為空 (長度留長一點，因為實作時一定要存 Hash 加密後的值)
    password VARCHAR(255) NOT NULL,
    
    -- Email：不可為空、不可重複
    email VARCHAR(255) UNIQUE NOT NULL,
    
    -- 其他選填的個人資訊
    name VARCHAR(100),
    shipping_address TEXT,
    recipient_name VARCHAR(100),
    phone VARCHAR(20),
    
    -- 系統自動記錄的時間戳記 (包含時區)
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);