-- TODO: add migration
-- 00017_create_refunds.sql
-- +goose Up
CREATE TYPE refund_status AS ENUM ('pending', 'approved', 'processed', 'failed');

CREATE TABLE refunds (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  transaction_id UUID NOT NULL REFERENCES transactions(id),
  approved_by   UUID REFERENCES users(id),
  amount        NUMERIC(12,2) NOT NULL,
  reason        TEXT,
  status        refund_status NOT NULL DEFAULT 'pending',
  mpesa_receipt TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  processed_at  TIMESTAMPTZ
);

CREATE INDEX idx_refunds_transaction_id ON refunds(transaction_id);

-- +goose Down
DROP TABLE refunds;
DROP TYPE refund_status;
