-- TODO: add migration
-- 00011_create_discount_codes.sql
-- +goose Up
CREATE TYPE discount_type AS ENUM ('percent', 'fixed');

CREATE TABLE discount_codes (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code              TEXT NOT NULL UNIQUE,
  type              discount_type NOT NULL,
  value             NUMERIC(12,2) NOT NULL,
  min_order_amount  NUMERIC(12,2) NOT NULL DEFAULT 0,
  usage_limit       INTEGER,
  used_count        INTEGER NOT NULL DEFAULT 0,
  expires_at        TIMESTAMPTZ,
  is_active         BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_discount_codes_code ON discount_codes(code);

-- +goose Down
DROP TABLE discount_codes;
DROP TYPE discount_type;
