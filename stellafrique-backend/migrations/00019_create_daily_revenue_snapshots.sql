-- TODO: add migration
-- 00019_create_daily_revenue_snapshots.sql
-- +goose Up
CREATE TABLE daily_revenue_snapshots (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  snapshot_date    DATE NOT NULL UNIQUE,
  total_revenue    NUMERIC(12,2) NOT NULL DEFAULT 0,
  total_orders     INTEGER NOT NULL DEFAULT 0,
  new_customers    INTEGER NOT NULL DEFAULT 0,
  units_sold       INTEGER NOT NULL DEFAULT 0,
  avg_order_value  NUMERIC(12,2) NOT NULL DEFAULT 0
);

CREATE INDEX idx_snapshots_date ON daily_revenue_snapshots(snapshot_date DESC);

-- +goose Down
DROP TABLE daily_revenue_snapshots;
