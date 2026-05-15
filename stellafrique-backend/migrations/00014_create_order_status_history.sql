-- TODO: add migration
-- 00014_create_order_status_history.sql
-- +goose Up
CREATE TABLE order_status_history (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id     UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  changed_by   UUID REFERENCES users(id),
  from_status  order_status,
  to_status    order_status NOT NULL,
  note         TEXT,
  changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_status_history_order_id ON order_status_history(order_id);

-- +goose Down
DROP TABLE order_status_history;
