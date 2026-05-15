use axum::{extract::State, routing::get, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{
    entities::{product_images, product_variants, products},
    error::AppError,
    state::AppState,
};

#[derive(Serialize)]
pub struct CatalogBlueprintResponse {
    collections: Vec<CollectionShape>,
}

#[derive(Serialize)]
pub struct CollectionShape {
    name: &'static str,
    purpose: &'static str,
}

#[derive(Serialize)]
pub struct FeaturedCatalogResponse {
    products: Vec<FeaturedProduct>,
}

#[derive(Serialize)]
pub struct FeaturedProduct {
    id: sea_orm::prelude::Uuid,
    category_id: Option<sea_orm::prelude::Uuid>,
    name: String,
    slug: String,
    short_description: Option<String>,
    price: Option<sea_orm::prelude::Decimal>,
    primary_image_url: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/catalog/blueprint", get(catalog_blueprint))
        .route("/catalog/featured", get(featured_products))
}

async fn catalog_blueprint() -> Json<CatalogBlueprintResponse> {
    Json(CatalogBlueprintResponse {
        collections: vec![
            CollectionShape {
                name: "categories",
                purpose: "Storefront grouping such as dresses, tops, or sets.",
            },
            CollectionShape {
                name: "products",
                purpose: "Core sellable concepts shown on listing and detail pages.",
            },
            CollectionShape {
                name: "product_variants",
                purpose: "Size, color, price, sku, and stock per purchasable option.",
            },
            CollectionShape {
                name: "product_images",
                purpose: "Ordered product and variant imagery for the storefront gallery.",
            },
        ],
    })
}

async fn featured_products(
    State(state): State<AppState>,
) -> Result<Json<FeaturedCatalogResponse>, AppError> {
    let featured = products::Entity::find()
        .filter(products::Column::Status.eq("active"))
        .filter(products::Column::IsFeatured.eq(true))
        .all(&state.db)
        .await?;

    let mut items = Vec::with_capacity(featured.len());

    for product in featured {
        let default_variant = product_variants::Entity::find()
            .filter(product_variants::Column::ProductId.eq(product.id))
            .filter(product_variants::Column::IsActive.eq(true))
            .one(&state.db)
            .await?;

        let primary_image = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(product.id))
            .filter(product_images::Column::IsPrimary.eq(true))
            .one(&state.db)
            .await?;

        items.push(FeaturedProduct {
            id: product.id,
            category_id: product.category_id,
            name: product.name,
            slug: product.slug,
            short_description: product.short_description,
            price: default_variant.map(|variant| variant.price),
            primary_image_url: primary_image.map(|image| image.image_url),
        });
    }

    Ok(Json(FeaturedCatalogResponse { products: items }))
}
