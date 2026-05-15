use axum::{extract::State, routing::get, Json, Router};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::{
    entities::{product_images, product_variants, products, store_settings},
    error::AppError,
    state::AppState,
};

#[derive(Serialize)]
pub struct HomeResponse {
    pub store: StoreSummary,
    pub featured_products: Vec<FeaturedProduct>,
}

#[derive(Serialize)]
pub struct StoreSummary {
    pub store_name: String,
    pub currency: String,
    pub delivery_fee: String,
    pub paybill_number: String,
}

#[derive(Serialize)]
pub struct FeaturedProduct {
    pub id: sea_orm::prelude::Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub default_price: Option<sea_orm::prelude::Decimal>,
    pub primary_image_url: Option<String>,
    pub available_sizes: Vec<String>,
    pub available_colors: Vec<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/storefront/home", get(get_home))
}

async fn get_home(State(state): State<AppState>) -> Result<Json<HomeResponse>, AppError> {
    let settings_rows = store_settings::Entity::find().all(&state.db).await?;
    let featured = products::Entity::find()
        .filter(products::Column::IsActive.eq(true))
        .filter(products::Column::IsFeatured.eq(true))
        .all(&state.db)
        .await?;

    let mut featured_products = Vec::with_capacity(featured.len());

    for product in featured {
        let variants = product_variants::Entity::find()
            .filter(product_variants::Column::ProductId.eq(product.id))
            .filter(product_variants::Column::IsActive.eq(true))
            .all(&state.db)
            .await?;

        let primary_image = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(product.id))
            .filter(product_images::Column::IsPrimary.eq(true))
            .one(&state.db)
            .await?;

        let default_price = variants.first().map(|variant| variant.price);
        let available_sizes = variants
            .iter()
            .filter_map(|variant| variant.size.clone())
            .collect();
        let available_colors = variants
            .iter()
            .filter_map(|variant| variant.color.clone())
            .collect();

        featured_products.push(FeaturedProduct {
            id: product.id,
            name: product.name,
            slug: product.slug,
            description: product.description,
            brand: product.brand,
            default_price,
            primary_image_url: primary_image.map(|image| image.url),
            available_sizes,
            available_colors,
        });
    }

    Ok(Json(HomeResponse {
        store: StoreSummary::from_settings(settings_rows),
        featured_products,
    }))
}

impl StoreSummary {
    fn from_settings(settings: Vec<store_settings::Model>) -> Self {
        let mut summary = Self {
            store_name: "Stellafrique".to_string(),
            currency: "KES".to_string(),
            delivery_fee: "0".to_string(),
            paybill_number: String::new(),
        };

        for item in settings {
            match item.key.as_str() {
                "store_name" => summary.store_name = item.value,
                "currency" => summary.currency = item.value,
                "delivery_fee" => summary.delivery_fee = item.value,
                "paybill_number" => summary.paybill_number = item.value,
                _ => {}
            }
        }

        summary
    }
}
