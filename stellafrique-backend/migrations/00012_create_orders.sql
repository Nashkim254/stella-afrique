-- TODO: add migration
-- 00012_create_orders.sql
-- +goose Up
CREATE TYPE order_status AS ENUM (
  'pending', 'confirmed', 'processing',
  'shipped', 'delivered', 'cancelled', 'refunded'
);

CREATE TABLE orders (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id           UUID NOT NULL REFERENCES users(id),
  address_id        UUID REFERENCES addresses(id),
  discount_code_id  UUID REFERENCES discount_codes(id),
  status            order_status NOT NULL DEFAULT 'pending',
  order_number      TEXT NOT NULL UNIQUE,
  subtotal          NUMERIC(12,2) NOT NULL,
  discount_amount   NUMERIC(12,2) NOT NULL DEFAULT 0,
  delivery_fee      NUMERIC(12,2) NOT NULL DEFAULT 0,
  total             NUMERIC(12,2) NOT NULL,
  notes             TEXT,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_user_id ON orders(user_id, created_at DESC);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_order_number ON orders(order_number);

-- +goose Down
DROP TABLE orders;
DROP TYPE order_status;
