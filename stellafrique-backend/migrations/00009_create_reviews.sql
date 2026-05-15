-- TODO: add migration
-- 00009_create_reviews.sql
-- +goose Up
CREATE TABLE reviews (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id     UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  order_item_id  UUID,
  rating         INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
  comment        TEXT,
  is_verified    BOOLEAN NOT NULL DEFAULT false,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reviews_product_id ON reviews(product_id);
CREATE UNIQUE INDEX idx_reviews_one_per_item ON reviews(user_id, order_item_id);

-- +goose Down
DROP TABLE reviews;
