-- TODO: add migration
-- 00001_create_users.sql
-- +goose Up
CREATE TYPE user_role AS ENUM ('customer', 'admin');

CREATE TABLE users (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  full_name        TEXT NOT NULL,
  email            TEXT NOT NULL UNIQUE,
  phone            TEXT,
  password_hash    TEXT NOT NULL,
  role             user_role NOT NULL DEFAULT 'customer',
  is_active        BOOLEAN NOT NULL DEFAULT true,
  avatar_url       TEXT,
  last_login_at    TIMESTAMPTZ,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_phone ON users(phone);

-- +goose Down
DROP TABLE users;
DROP TYPE user_role;
