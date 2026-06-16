# Stellafrique

Split ecommerce codebase for a fashion storefront:

- `stellafrique-backend`: Rust API using Axum, SeaORM, and Postgres.
- `stellafrique-backend/migration`: SeaORM migration crate for the catalog schema.
- `stellafrique-frontend`: Nuxt storefront shell ready for Figma-driven UI work.

## Local setup

1. Start Postgres and create a `stellafrique` database.
2. Create `stellafrique-backend/.env` and set `DATABASE_URL`.
3. Apply the initial schema with `cargo run -p migration -- up`.
4. Seed the storefront catalog with `cargo run --bin seed_catalog`.
5. Run the backend with `cargo run -p stellafrique-backend`.
6. Install frontend dependencies in `stellafrique-frontend` with `npm install`.
7. Start Nuxt with `node ./node_modules/@nuxt/cli/bin/nuxi.mjs dev`.

To enable product image uploads to Supabase Storage, also set these backend variables in `stellafrique-backend/.env`:

```env
SUPABASE_PROJECT_URL=https://YOUR_PROJECT_REF.supabase.co
SUPABASE_STORAGE_BUCKET=product-images
SUPABASE_SERVICE_ROLE_KEY=YOUR_SERVICE_ROLE_KEY
```

To enable transactional emails through Resend, also set these backend variables in `stellafrique-backend/.env`:

```env
RESEND_API_KEY=re_...
RESEND_FROM_EMAIL=orders@yourdomain.com
RESEND_FROM_NAME=Stellafrique
RESEND_REPLY_TO=support@yourdomain.com
RESEND_NOTIFICATION_EMAILS=owner@yourdomain.com,ops@yourdomain.com
FRONTEND_ORIGIN=http://localhost:3001
ADMIN_EMAIL=admin@stellafrique.com
ADMIN_PASSWORD=choose-a-strong-password
ADMIN_SESSION_SECRET=choose-a-long-random-secret
CUSTOMER_SESSION_SECRET=choose-a-second-long-random-secret
```

`RESEND_FROM_EMAIL` must be a verified sender or come from a verified domain in your Resend account.
`RESEND_NOTIFICATION_EMAILS` is optional and sends internal store-team emails for new orders and order state changes.
`FRONTEND_ORIGIN` should match the Nuxt app origin so cookie-based admin auth works in the browser.
`ADMIN_EMAIL`, `ADMIN_PASSWORD`, and `ADMIN_SESSION_SECRET` are required if you want `/admin/*` protected by login.
`CUSTOMER_SESSION_SECRET` is required for customer register/login/account sessions.

After adding customer auth and order linkage, apply migrations again:

```bash
cd stellafrique-backend
cargo run -p migration -- up
```

## Current API

- `GET /api/v1/health`
- `GET /api/v1/catalog/blueprint`
- `GET /api/v1/catalog/featured`
- `GET /api/v1/catalog/products`
- `GET /api/v1/catalog/collections/:slug`
- `GET /api/v1/catalog/products/:slug`

## Catalog Seeding

The backend now includes a repeatable development seed command:

```bash
cd stellafrique-backend
cargo run --bin seed_catalog
```

This command clears and repopulates the catalog tables:
- `categories`
- `products`
- `product_variants`
- `product_images`

It is intended for local development while the storefront and admin flows are still evolving.
