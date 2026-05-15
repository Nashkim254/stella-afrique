# Stellafrique

Split ecommerce codebase for a fashion storefront:

- `stellafrique-backend`: Rust API using Axum, SeaORM, and Postgres.
- `stellafrique-frontend`: Nuxt storefront shell ready for Figma-driven UI work.

## Local setup

1. Start Postgres and create a `stellafrique` database.
2. Copy `stellafrique-backend/.env.example` to `.env` and update `DATABASE_URL`.
3. Apply the SQL migrations in `stellafrique-backend/migrations`.
4. Run the backend with `cargo run`.
5. Install frontend dependencies in `stellafrique-frontend` with `npm install`, then start Nuxt with `npm run dev`.

## Current API

- `GET /api/v1/health`
- `GET /api/v1/storefront/home`

