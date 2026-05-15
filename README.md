# Stellafrique

Split ecommerce codebase for a fashion storefront:

- `stellafrique-backend`: Rust API using Axum, SeaORM, and Postgres.
- `stellafrique-backend/migration`: SeaORM migration crate for the catalog schema.
- `stellafrique-frontend`: Nuxt storefront shell ready for Figma-driven UI work.

## Local setup

1. Start Postgres and create a `stellafrique` database.
2. Copy `stellafrique-backend/.env.example` to `.env` and update `DATABASE_URL`.
3. Apply the initial schema with `cargo run -p migration -- up`.
4. Run the backend with `cargo run -p stellafrique-backend`.
5. Install frontend dependencies in `stellafrique-frontend` with `npm install`.
6. Start Nuxt with `node ./node_modules/@nuxt/cli/bin/nuxi.mjs dev`.

## Current API

- `GET /api/v1/health`
- `GET /api/v1/catalog/blueprint`
- `GET /api/v1/catalog/featured`
