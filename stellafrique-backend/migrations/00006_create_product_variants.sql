-- TODO: add migration
-- 00006_create_product_variants.sql
-- +goose Up
CREATE TABLE product_variants (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id           UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  size                 TEXT,
  color                TEXT,
  color_hex            TEXT,
  price                NUMERIC(12,2) NOT NULL,
  compare_at_price     NUMERIC(12,2),
  stock_qty            INTEGER NOT NULL DEFAULT 0,
  low_stock_threshold  INTEGER NOT NULL DEFAULT 5,
  sku                  TEXT NOT NULL UNIQUE,
  is_active            BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_variants_product_id ON product_variants(product_id);
CREATE INDEX idx_variants_sku ON product_variants(sku);
CREATE INDEX idx_variants_low_stock ON product_variants(stock_qty, low_stock_threshold);

-- +goose Down
DROP TABLE product_variants;
