-- Add up migration script here
-- Add up migration script here
-- Add migration script here
CREATE TABLE refresh_tokens (
    -- 主鍵：使用refresh token直接作為主鍵
    id UUID DEFAULT uuidv7(),
    
    token VARCHAR(255) PRIMARY KEY NOT NULL,
    
    -- 帳號：不可為空、不可重複，與user id外鍵
    user_id UUID NOT NULL references users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    
    
    -- token過期時間
    expires_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- token建立時間
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);