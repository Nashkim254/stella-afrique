-- TODO: add migration
-- 00003_create_addresses.sql
-- +goose Up
CREATE TABLE addresses (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  label       TEXT,
  county      TEXT NOT NULL,
  town        TEXT NOT NULL,
  street      TEXT,
  landmark    TEXT,
  is_default  BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX idx_addresses_user_id ON addresses(user_id);

-- +goose Down
DROP TABLE addresses;
