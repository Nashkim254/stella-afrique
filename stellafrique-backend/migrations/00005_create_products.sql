-- TODO: add migration
-- 00005_create_products.sql
-- +goose Up
CREATE TABLE products (
  id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  category_id        UUID REFERENCES categories(id) ON DELETE SET NULL,
  name               TEXT NOT NULL,
  slug               TEXT NOT NULL UNIQUE,
  description        TEXT,
  care_instructions  TEXT,
  brand              TEXT,
  is_active          BOOLEAN NOT NULL DEFAULT true,
  is_featured        BOOLEAN NOT NULL DEFAULT false,
  avg_rating         NUMERIC(3,2) NOT NULL DEFAULT 0,
  review_count       INTEGER NOT NULL DEFAULT 0,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_slug ON products(slug);
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_products_is_featured ON products(is_featured) WHERE is_featured = true;

-- +goose Down
DROP TABLE products;
