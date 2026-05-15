-- TODO: add migration
-- 00008_create_product_tags.sql
-- +goose Up
CREATE TABLE product_tags (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id  UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  tag         TEXT NOT NULL
);

CREATE INDEX idx_tags_product_id ON product_tags(product_id);
CREATE INDEX idx_tags_tag ON product_tags(tag);

-- +goose Down
DROP TABLE product_tags;
