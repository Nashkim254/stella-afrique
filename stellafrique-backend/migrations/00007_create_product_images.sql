-- TODO: add migration
-- 00007_create_product_images.sql
-- +goose Up
CREATE TABLE product_images (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id  UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  variant_id  UUID REFERENCES product_variants(id) ON DELETE SET NULL,
  public_id   TEXT NOT NULL,
  url         TEXT NOT NULL,
  alt_text    TEXT,
  is_primary  BOOLEAN NOT NULL DEFAULT false,
  sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_images_product_id ON product_images(product_id);

-- +goose Down
DROP TABLE product_images;
