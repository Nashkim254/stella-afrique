-- TODO: add migration
-- 00013_create_order_items.sql
-- +goose Up
CREATE TABLE order_items (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id       UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  variant_id     UUID REFERENCES product_variants(id) ON DELETE SET NULL,
  quantity       INTEGER NOT NULL CHECK (quantity > 0),
  unit_price     NUMERIC(12,2) NOT NULL,
  snapshot_name  TEXT NOT NULL,
  snapshot_sku   TEXT NOT NULL
);

CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_variant_id ON order_items(variant_id);

-- +goose Down
DROP TABLE order_items;
