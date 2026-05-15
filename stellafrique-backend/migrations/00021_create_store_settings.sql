-- TODO: add migration
-- 00021_create_store_settings.sql
-- +goose Up
CREATE TABLE store_settings (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  key         TEXT NOT NULL UNIQUE,
  value       TEXT NOT NULL,
  updated_by  TEXT,
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO store_settings (key, value) VALUES
  ('store_name', 'Clothline'),
  ('delivery_fee', '200'),
  ('paybill_number', ''),
  ('currency', 'KES');

-- +goose Down
DROP TABLE store_settings;
