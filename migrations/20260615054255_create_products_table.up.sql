-- Add up migration script here
CREATE TYPE product_status AS ENUM ('draft', 'published', 'archived');
CREATE TABLE products (


    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 基本資訊
    name VARCHAR(255) NOT NULL,
    description TEXT,
    specs JSONB,             -- 存放規格，例如 {"品牌": "Sony", "重量": "1kg"}
    images JSONB,            -- 存放圖片 URL 陣列
    
    -- 銷售與狀態
    status product_status NOT NULL DEFAULT 'draft', -- draft, published, archived
    original_price INT NOT NULL,
    selling_price INT NOT NULL,
    stock_available INT NOT NULL DEFAULT 0,      -- 只有這個庫存數字留在此表
    
    -- 營運設定
    shipping_methods JSONB,  -- 例如 ["宅配", "超商取貨"]
    notes TEXT,              -- 內部備註 (不顯示給前台客人的)
    
    -- 時間戳記
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);