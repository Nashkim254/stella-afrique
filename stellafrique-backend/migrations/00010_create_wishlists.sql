-- TODO: add migration
-- 00010_create_wishlists.sql
-- +goose Up
CREATE TABLE wishlists (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  variant_id  UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
  added_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(user_id, variant_id)
);

CREATE INDEX idx_wishlists_user_id ON wishlists(user_id);

-- +goose Down
DROP TABLE wishlists;
