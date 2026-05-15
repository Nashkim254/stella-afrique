-- TODO: add migration
-- 00015_create_delivery_tracking.sql
-- +goose Up
CREATE TABLE delivery_tracking (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id         UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  courier          TEXT,
  tracking_number  TEXT,
  status           TEXT,
  location_note    TEXT,
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_delivery_order_id ON delivery_tracking(order_id);

-- +goose Down
DROP TABLE delivery_tracking;
