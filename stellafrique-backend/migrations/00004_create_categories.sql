-- TODO: add migration
-- 00004_create_categories.sql
-- +goose Up
CREATE TABLE categories (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  parent_id   UUID REFERENCES categories(id) ON DELETE SET NULL,
  name        TEXT NOT NULL,
  slug        TEXT NOT NULL UNIQUE,
  image_url   TEXT,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  is_active   BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_categories_slug ON categories(slug);
CREATE INDEX idx_categories_parent_id ON categories(parent_id);

-- +goose Down
DROP TABLE categories;
