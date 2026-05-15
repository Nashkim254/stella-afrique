-- TODO: add migration
-- 00016_create_transactions.sql
-- +goose Up
CREATE TYPE txn_method AS ENUM ('stk_push', 'paybill');
CREATE TYPE txn_status AS ENUM ('initiated', 'pending', 'success', 'failed', 'timeout');

CREATE TABLE transactions (
  id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id              UUID NOT NULL REFERENCES orders(id),
  method                txn_method NOT NULL,
  status                txn_status NOT NULL DEFAULT 'initiated',
  amount                NUMERIC(12,2) NOT NULL,
  checkout_request_id   TEXT,
  mpesa_receipt         TEXT,
  phone_hash            TEXT,
  paybill_number        TEXT,
  failure_reason        TEXT,
  initiated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  completed_at          TIMESTAMPTZ
);

CREATE INDEX idx_transactions_order_id ON transactions(order_id);
CREATE INDEX idx_transactions_checkout_request_id ON transactions(checkout_request_id);
CREATE INDEX idx_transactions_status ON transactions(status);

-- +goose Down
DROP TABLE transactions;
DROP TYPE txn_method;
DROP TYPE txn_status;
