use axum::{
    http::StatusCode,
    extract::Multipart,
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, TransactionTrait};
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::{
    entities::{categories, inventory_events, order_items, product_images, product_variants, products},
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
    category: Option<String>,
    short_description: Option<String>,
    price: Option<sea_orm::prelude::Decimal>,
    primary_image_url: Option<String>,
}

#[derive(Deserialize)]
pub struct CatalogProductsQuery {
    collection: Option<String>,
    search: Option<String>,
    sort: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CatalogProductCard {
    id: sea_orm::prelude::Uuid,
    slug: String,
    name: String,
    category: Option<String>,
    category_slug: Option<String>,
    short_description: Option<String>,
    price: Option<sea_orm::prelude::Decimal>,
    primary_image_url: Option<String>,
}

#[derive(Serialize)]
pub struct CatalogProductsResponse {
    products: Vec<CatalogProductCard>,
}

#[derive(Serialize)]
pub struct AdminCatalogOverviewResponse {
    categories: Vec<AdminCategoryRecord>,
    products: Vec<AdminProductRecord>,
}

#[derive(Serialize)]
pub struct AdminCategoryRecord {
    id: sea_orm::prelude::Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    image_url: Option<String>,
    sort_order: i32,
    is_active: bool,
}

#[derive(Serialize)]
pub struct AdminProductRecord {
    id: sea_orm::prelude::Uuid,
    name: String,
    slug: String,
    category: Option<String>,
    category_slug: Option<String>,
    short_description: Option<String>,
    description: Option<String>,
    status: String,
    is_featured: bool,
}

#[derive(Serialize)]
pub struct AdminProductDetail {
    id: sea_orm::prelude::Uuid,
    name: String,
    slug: String,
    category: Option<String>,
    category_slug: Option<String>,
    short_description: Option<String>,
    description: Option<String>,
    status: String,
    is_featured: bool,
    primary_image_url: Option<String>,
    images: Vec<ProductImageAsset>,
    variants: Vec<ProductVariantOption>,
}

#[derive(Serialize)]
pub struct AdminInventoryRecord {
    product_name: String,
    product_slug: String,
    variant_id: sea_orm::prelude::Uuid,
    variant_name: String,
    sku: String,
    size: Option<String>,
    color: Option<String>,
    stock_quantity: i32,
    is_active: bool,
    latest_event: Option<InventoryEventSummary>,
}

#[derive(Serialize, Clone)]
pub struct InventoryEventSummary {
    event_type: String,
    actor: String,
    message: String,
    reason: String,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Serialize, Clone)]
pub struct InventoryEventRecord {
    event_type: String,
    actor: String,
    message: String,
    reason: String,
    previous_stock_quantity: Option<i32>,
    next_stock_quantity: Option<i32>,
    previous_is_active: Option<bool>,
    next_is_active: Option<bool>,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Deserialize)]
pub struct UpdateInventoryPayload {
    stock_quantity: Option<i32>,
    is_active: Option<bool>,
    adjustment_reason: Option<String>,
}

#[derive(Serialize)]
pub struct CatalogProductDetail {
    id: sea_orm::prelude::Uuid,
    slug: String,
    name: String,
    category: Option<String>,
    category_slug: Option<String>,
    short_description: Option<String>,
    description: Option<String>,
    status: String,
    is_featured: bool,
    images: Vec<ProductImageAsset>,
    variants: Vec<ProductVariantOption>,
}

#[derive(Serialize)]
pub struct ProductImageAsset {
    image_url: String,
    alt_text: Option<String>,
    is_primary: bool,
    sort_order: i32,
}

#[derive(Serialize)]
pub struct ProductVariantOption {
    id: sea_orm::prelude::Uuid,
    name: String,
    sku: String,
    size: Option<String>,
    color: Option<String>,
    price: sea_orm::prelude::Decimal,
    compare_at_price: Option<sea_orm::prelude::Decimal>,
    stock_quantity: i32,
    is_active: bool,
}

#[derive(Serialize)]
pub struct AdminUploadResponse {
    object_path: String,
    public_url: String,
}

#[derive(Deserialize)]
pub struct CreateCategoryPayload {
    name: String,
    slug: String,
    description: Option<String>,
    image_url: Option<String>,
    sort_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryPayload {
    name: Option<String>,
    slug: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    sort_order: Option<i32>,
    is_active: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateProductVariantPayload {
    id: Option<sea_orm::prelude::Uuid>,
    name: Option<String>,
    sku: String,
    size: Option<String>,
    color: Option<String>,
    price: String,
    compare_at_price: Option<String>,
    stock_quantity: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateProductPayload {
    category_slug: Option<String>,
    name: String,
    slug: String,
    short_description: Option<String>,
    description: Option<String>,
    status: Option<String>,
    is_featured: Option<bool>,
    primary_image_url: Option<String>,
    gallery_image_urls: Option<Vec<String>>,
    price: String,
    compare_at_price: Option<String>,
    sku: String,
    variant_name: Option<String>,
    size: Option<String>,
    color: Option<String>,
    stock_quantity: Option<i32>,
    variants: Option<Vec<CreateProductVariantPayload>>,
}

#[derive(Deserialize)]
pub struct UpdateProductPayload {
    category_slug: Option<String>,
    name: Option<String>,
    slug: Option<String>,
    short_description: Option<String>,
    description: Option<String>,
    status: Option<String>,
    is_featured: Option<bool>,
    primary_image_url: Option<String>,
    gallery_image_urls: Option<Vec<String>>,
    variants: Option<Vec<CreateProductVariantPayload>>,
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/catalog/blueprint", get(catalog_blueprint))
        .route("/catalog/featured", get(featured_products))
        .route("/catalog/products", get(list_products))
        .route("/catalog/collections/:slug", get(list_products_by_collection))
        .route("/catalog/products/:slug", get(product_detail))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/catalog/overview", get(admin_catalog_overview))
        .route("/admin/products/:id", get(admin_product_detail).patch(update_product).delete(delete_product))
        .route("/admin/inventory", get(admin_inventory_overview))
        .route(
            "/admin/inventory/:variant_id",
            get(inventory_history).patch(update_inventory_record),
        )
        .route("/admin/categories", post(create_category))
        .route("/admin/categories/:id", patch(update_category).delete(delete_category))
        .route("/admin/products", post(create_product))
        .route("/admin/uploads/product-image", post(upload_product_image))
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

        let category = match product.category_id {
            Some(category_id) => categories::Entity::find_by_id(category_id)
                .one(&state.db)
                .await?
                .map(|record| record.name),
            None => None,
        };

        items.push(FeaturedProduct {
            id: product.id,
            category_id: product.category_id,
            name: product.name,
            slug: product.slug,
            category,
            short_description: product.short_description,
            price: default_variant.map(|variant| variant.price),
            primary_image_url: primary_image.map(|image| image.image_url),
        });
    }

    Ok(Json(FeaturedCatalogResponse { products: items }))
}

async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<CatalogProductsQuery>,
) -> Result<Json<CatalogProductsResponse>, AppError> {
    let products = fetch_catalog_products(&state, query.collection, query.search, query.sort).await?;
    Ok(Json(CatalogProductsResponse { products }))
}

async fn list_products_by_collection(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<CatalogProductsQuery>,
) -> Result<Json<CatalogProductsResponse>, AppError> {
    let products = fetch_catalog_products(&state, Some(slug), query.search, query.sort).await?;
    Ok(Json(CatalogProductsResponse { products }))
}

async fn product_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<CatalogProductDetail>, AppError> {
    let product = products::Entity::find()
        .filter(products::Column::Slug.eq(slug))
        .filter(products::Column::Status.eq("active"))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let category = match product.category_id {
        Some(category_id) => categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await?,
        None => None,
    };

    let variants = product_variants::Entity::find()
        .filter(product_variants::Column::ProductId.eq(product.id))
        .filter(product_variants::Column::IsActive.eq(true))
        .order_by_asc(product_variants::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let images = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(product.id))
        .order_by_desc(product_images::Column::IsPrimary)
        .order_by_asc(product_images::Column::SortOrder)
        .all(&state.db)
        .await?;

    Ok(Json(CatalogProductDetail {
        id: product.id,
        slug: product.slug,
        name: product.name,
        category: category.as_ref().map(|record| record.name.clone()),
        category_slug: category.as_ref().map(|record| record.slug.clone()),
        short_description: product.short_description,
        description: product.description,
        status: product.status,
        is_featured: product.is_featured,
        images: images
            .into_iter()
            .map(|image| ProductImageAsset {
                image_url: image.image_url,
                alt_text: image.alt_text,
                is_primary: image.is_primary,
                sort_order: image.sort_order,
            })
            .collect(),
        variants: variants
            .into_iter()
            .map(|variant| ProductVariantOption {
                id: variant.id,
                name: variant.name,
                sku: variant.sku,
                size: variant.size,
                color: variant.color,
                price: variant.price,
                compare_at_price: variant.compare_at_price,
                stock_quantity: variant.stock_quantity,
                is_active: variant.is_active,
            })
            .collect(),
    }))
}

async fn admin_catalog_overview(
    State(state): State<AppState>,
) -> Result<Json<AdminCatalogOverviewResponse>, AppError> {
    let category_rows = categories::Entity::find()
        .order_by_asc(categories::Column::SortOrder)
        .all(&state.db)
        .await?;

    let product_rows = products::Entity::find()
        .order_by_desc(products::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let mut admin_products = Vec::with_capacity(product_rows.len());
    for product in product_rows {
        let category_record = match product.category_id {
            Some(category_id) => categories::Entity::find_by_id(category_id)
                .one(&state.db)
                .await?,
            None => None,
        };

        admin_products.push(AdminProductRecord {
            id: product.id,
            name: product.name,
            slug: product.slug,
            category: category_record.as_ref().map(|record| record.name.clone()),
            category_slug: category_record.as_ref().map(|record| record.slug.clone()),
            short_description: product.short_description,
            description: product.description,
            status: product.status,
            is_featured: product.is_featured,
        });
    }

    Ok(Json(AdminCatalogOverviewResponse {
        categories: category_rows
            .into_iter()
            .map(|category| AdminCategoryRecord {
                id: category.id,
                name: category.name,
                slug: category.slug,
                description: category.description,
                image_url: category.image_url,
                sort_order: category.sort_order,
                is_active: category.is_active,
            })
            .collect(),
        products: admin_products,
    }))
}

async fn admin_product_detail(
    State(state): State<AppState>,
    Path(product_id): Path<sea_orm::prelude::Uuid>,
) -> Result<Json<AdminProductDetail>, AppError> {
    let product = products::Entity::find_by_id(product_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(build_admin_product_detail(&state, product).await?))
}

async fn admin_inventory_overview(
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminInventoryRecord>>, AppError> {
    let variants = product_variants::Entity::find()
        .order_by_asc(product_variants::Column::StockQuantity)
        .order_by_asc(product_variants::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let mut inventory = Vec::with_capacity(variants.len());
    for variant in variants {
        let product = products::Entity::find_by_id(variant.product_id)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;
        let latest_event = latest_inventory_event(&state, variant.id).await?;

        inventory.push(AdminInventoryRecord {
            product_name: product.name,
            product_slug: product.slug,
            variant_id: variant.id,
            variant_name: variant.name,
            sku: variant.sku,
            size: variant.size,
            color: variant.color,
            stock_quantity: variant.stock_quantity,
            is_active: variant.is_active,
            latest_event,
        });
    }

    Ok(Json(inventory))
}

async fn update_inventory_record(
    State(state): State<AppState>,
    Path(variant_id): Path<sea_orm::prelude::Uuid>,
    Json(payload): Json<UpdateInventoryPayload>,
) -> Result<Json<AdminInventoryRecord>, AppError> {
    let variant = product_variants::Entity::find_by_id(variant_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let product = products::Entity::find_by_id(variant.product_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let next_stock_quantity = payload.stock_quantity.unwrap_or(variant.stock_quantity);
    let next_is_active = payload.is_active.unwrap_or(variant.is_active);

    if next_stock_quantity < 0 {
        return Err(AppError::BadRequest("stock quantity cannot be negative".to_owned()));
    }

    if next_stock_quantity == variant.stock_quantity && next_is_active == variant.is_active {
        return Err(AppError::BadRequest("no inventory changes were provided".to_owned()));
    }

    let adjustment_reason = payload
        .adjustment_reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::BadRequest("adjustment reason is required".to_owned()))?
        .to_owned();

    let previous_stock_quantity = variant.stock_quantity;
    let previous_is_active = variant.is_active;
    let transaction = state.db.begin().await?;
    let mut active_variant = product_variants::ActiveModel::from(variant.clone());
    active_variant.stock_quantity = Set(next_stock_quantity);
    active_variant.is_active = Set(next_is_active);
    active_variant.updated_at = Set(Utc::now().fixed_offset());
    let variant = active_variant.update(&transaction).await?;

    inventory_events::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        product_id: Set(product.id),
        variant_id: Set(variant.id),
        event_type: Set("manual_adjustment".to_owned()),
        actor: Set("admin".to_owned()),
        message: Set(build_inventory_event_message(
            previous_stock_quantity,
            previous_is_active,
            next_stock_quantity,
            next_is_active,
            &variant.sku,
        )),
        reason: Set(adjustment_reason),
        previous_stock_quantity: Set(Some(previous_stock_quantity)),
        next_stock_quantity: Set(Some(next_stock_quantity)),
        previous_is_active: Set(Some(previous_is_active)),
        next_is_active: Set(Some(next_is_active)),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(&transaction)
    .await?;

    transaction.commit().await?;
    let latest_event = latest_inventory_event(&state, variant.id).await?;

    Ok(Json(AdminInventoryRecord {
        product_name: product.name,
        product_slug: product.slug,
        variant_id: variant.id,
        variant_name: variant.name,
        sku: variant.sku,
        size: variant.size,
        color: variant.color,
        stock_quantity: variant.stock_quantity,
        is_active: variant.is_active,
        latest_event,
    }))
}

async fn inventory_history(
    State(state): State<AppState>,
    Path(variant_id): Path<sea_orm::prelude::Uuid>,
) -> Result<Json<Vec<InventoryEventRecord>>, AppError> {
    let records = inventory_events::Entity::find()
        .filter(inventory_events::Column::VariantId.eq(variant_id))
        .order_by_desc(inventory_events::Column::CreatedAt)
        .all(&state.db)
        .await?;

    Ok(Json(
        records
            .into_iter()
            .map(|event| InventoryEventRecord {
                event_type: event.event_type,
                actor: event.actor,
                message: event.message,
                reason: event.reason,
                previous_stock_quantity: event.previous_stock_quantity,
                next_stock_quantity: event.next_stock_quantity,
                previous_is_active: event.previous_is_active,
                next_is_active: event.next_is_active,
                created_at: event.created_at,
            })
            .collect(),
    ))
}

async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<(StatusCode, Json<AdminCategoryRecord>), AppError> {
    let now = Utc::now().fixed_offset();
    let category = categories::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        name: Set(payload.name),
        slug: Set(payload.slug),
        description: Set(payload.description),
        image_url: Set(payload.image_url.clone()),
        is_active: Set(true),
        sort_order: Set(payload.sort_order.unwrap_or(0)),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(AdminCategoryRecord {
            id: category.id,
            name: category.name,
            slug: category.slug,
            description: category.description,
            image_url: category.image_url,
            sort_order: category.sort_order,
            is_active: category.is_active,
        }),
    ))
}

async fn update_category(
    State(state): State<AppState>,
    Path(category_id): Path<sea_orm::prelude::Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> Result<Json<AdminCategoryRecord>, AppError> {
    if payload.name.is_none()
        && payload.slug.is_none()
        && payload.description.is_none()
        && payload.image_url.is_none()
        && payload.sort_order.is_none()
        && payload.is_active.is_none()
    {
        return Err(AppError::BadRequest("no category changes were provided".to_owned()));
    }

    let category = categories::Entity::find_by_id(category_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut active_category = categories::ActiveModel::from(category);
    if let Some(name) = payload.name {
        active_category.name = Set(name.trim().to_owned());
    }
    if let Some(slug) = payload.slug {
        active_category.slug = Set(slug.trim().to_owned());
    }
    if let Some(description) = payload.description {
        active_category.description = Set(if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_owned())
        });
    }
    if let Some(image_url) = payload.image_url {
        active_category.image_url = Set(if image_url.trim().is_empty() {
            None
        } else {
            Some(image_url.trim().to_owned())
        });
    }
    if let Some(sort_order) = payload.sort_order {
        active_category.sort_order = Set(sort_order);
    }
    if let Some(is_active) = payload.is_active {
        active_category.is_active = Set(is_active);
    }
    active_category.updated_at = Set(Utc::now().fixed_offset());
    let category = active_category.update(&state.db).await?;

    Ok(Json(AdminCategoryRecord {
        id: category.id,
        name: category.name,
        slug: category.slug,
        description: category.description,
        image_url: category.image_url,
        sort_order: category.sort_order,
        is_active: category.is_active,
    }))
}

async fn delete_category(
    State(state): State<AppState>,
    Path(category_id): Path<sea_orm::prelude::Uuid>,
) -> Result<StatusCode, AppError> {
    let category = categories::Entity::find_by_id(category_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    categories::Entity::delete_by_id(category.id).exec(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductPayload>,
) -> Result<(StatusCode, Json<CatalogProductDetail>), AppError> {
    let now = Utc::now().fixed_offset();
    let category = match payload.category_slug {
        Some(ref slug) if !slug.is_empty() => categories::Entity::find()
            .filter(categories::Column::Slug.eq(slug.clone()))
            .one(&state.db)
            .await?,
        _ => None,
    };

    let product_id = uuid::Uuid::new_v4();
    let product = products::ActiveModel {
        id: Set(product_id),
        category_id: Set(category.as_ref().map(|record| record.id)),
        name: Set(payload.name),
        slug: Set(payload.slug),
        short_description: Set(payload.short_description),
        description: Set(payload.description),
        status: Set(payload.status.unwrap_or_else(|| "active".to_owned())),
        is_featured: Set(payload.is_featured.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    let requested_variants = match payload.variants {
        Some(variants) if !variants.is_empty() => variants,
        _ => vec![CreateProductVariantPayload {
            id: None,
            name: payload.variant_name,
            sku: payload.sku,
            size: payload.size,
            color: payload.color,
            price: payload.price,
            compare_at_price: payload.compare_at_price,
            stock_quantity: payload.stock_quantity,
        }],
    };

    let mut created_variants = Vec::with_capacity(requested_variants.len());
    for requested_variant in requested_variants {
        let variant = product_variants::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            product_id: Set(product.id),
            name: Set(
                requested_variant
                    .name
                    .unwrap_or_else(|| "Default Variant".to_owned()),
            ),
            sku: Set(requested_variant.sku),
            size: Set(requested_variant.size),
            color: Set(requested_variant.color),
            price: Set(parse_decimal(&requested_variant.price)),
            compare_at_price: Set(
                requested_variant
                    .compare_at_price
                    .as_deref()
                    .map(parse_decimal),
            ),
            stock_quantity: Set(requested_variant.stock_quantity.unwrap_or(0)),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&state.db)
        .await?;

        created_variants.push(ProductVariantOption {
            id: variant.id,
            name: variant.name,
            sku: variant.sku,
            size: variant.size,
            color: variant.color,
            price: variant.price,
            compare_at_price: variant.compare_at_price,
            stock_quantity: variant.stock_quantity,
            is_active: variant.is_active,
        });
    }

    let mut image_urls = Vec::new();
    if let Some(primary_image_url) = payload.primary_image_url {
        let primary_image_url = primary_image_url.trim().to_string();
        if !primary_image_url.is_empty() {
            image_urls.push(primary_image_url);
        }
    }

    if let Some(gallery_image_urls) = payload.gallery_image_urls {
        for image_url in gallery_image_urls {
            let image_url = image_url.trim().to_string();
            if !image_url.is_empty() && !image_urls.contains(&image_url) {
                image_urls.push(image_url);
            }
        }
    }

    let mut images = Vec::with_capacity(image_urls.len());
    for (index, image_url) in image_urls.into_iter().enumerate() {
        let image = product_images::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            product_id: Set(product.id),
            variant_id: Set(None),
            image_url: Set(image_url),
            alt_text: Set(Some(product.name.clone())),
            sort_order: Set(index as i32),
            is_primary: Set(index == 0),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&state.db)
        .await?;

        images.push(ProductImageAsset {
            image_url: image.image_url,
            alt_text: image.alt_text,
            is_primary: image.is_primary,
            sort_order: image.sort_order,
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(CatalogProductDetail {
            id: product.id,
            slug: product.slug,
            name: product.name,
            category: category.as_ref().map(|record| record.name.clone()),
            category_slug: category.as_ref().map(|record| record.slug.clone()),
            short_description: product.short_description,
            description: product.description,
            status: product.status,
            is_featured: product.is_featured,
            images,
            variants: created_variants,
        }),
    ))
}

async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<sea_orm::prelude::Uuid>,
    Json(payload): Json<UpdateProductPayload>,
) -> Result<Json<AdminProductDetail>, AppError> {
    if payload.category_slug.is_none()
        && payload.name.is_none()
        && payload.slug.is_none()
        && payload.short_description.is_none()
        && payload.description.is_none()
        && payload.status.is_none()
        && payload.is_featured.is_none()
        && payload.primary_image_url.is_none()
        && payload.gallery_image_urls.is_none()
        && payload.variants.is_none()
    {
        return Err(AppError::BadRequest("no product changes were provided".to_owned()));
    }

    let product = products::Entity::find_by_id(product_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let category = match payload.category_slug.as_deref().map(str::trim) {
        Some("") => None,
        Some(slug) => categories::Entity::find()
            .filter(categories::Column::Slug.eq(slug.to_owned()))
            .one(&state.db)
            .await?,
        None => match product.category_id {
            Some(category_id) => categories::Entity::find_by_id(category_id).one(&state.db).await?,
            None => None,
        },
    };

    let now = Utc::now().fixed_offset();
    let transaction = state.db.begin().await?;
    let mut active_product = products::ActiveModel::from(product);
    if payload.category_slug.is_some() {
        active_product.category_id = Set(category.as_ref().map(|record| record.id));
    }
    if let Some(name) = payload.name {
        active_product.name = Set(name.trim().to_owned());
    }
    if let Some(slug) = payload.slug {
        active_product.slug = Set(slug.trim().to_owned());
    }
    if let Some(short_description) = payload.short_description {
        active_product.short_description = Set(if short_description.trim().is_empty() {
            None
        } else {
            Some(short_description.trim().to_owned())
        });
    }
    if let Some(description) = payload.description {
        active_product.description = Set(if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_owned())
        });
    }
    if let Some(status) = payload.status {
        active_product.status = Set(status.trim().to_lowercase());
    }
    if let Some(is_featured) = payload.is_featured {
        active_product.is_featured = Set(is_featured);
    }
    active_product.updated_at = Set(now);
    let product = active_product.update(&transaction).await?;

    if payload.primary_image_url.is_some() || payload.gallery_image_urls.is_some() {
        sync_product_images(
            &transaction,
            product.id,
            &product.name,
            payload.primary_image_url,
            payload.gallery_image_urls,
            now,
        )
        .await?;
    }

    if let Some(variants) = payload.variants {
        sync_product_variants(&transaction, product.id, variants, now).await?;
    }

    transaction.commit().await?;

    Ok(Json(build_admin_product_detail(&state, product).await?))
}

async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<sea_orm::prelude::Uuid>,
) -> Result<StatusCode, AppError> {
    let product = products::Entity::find_by_id(product_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let has_orders = order_items::Entity::find()
        .filter(order_items::Column::ProductId.eq(product.id))
        .one(&state.db)
        .await?
        .is_some();

    if has_orders {
        return Err(AppError::BadRequest(
            "cannot delete a product that already exists in customer orders".to_owned(),
        ));
    }

    products::Entity::delete_by_id(product.id).exec(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn latest_inventory_event(
    state: &AppState,
    variant_id: sea_orm::prelude::Uuid,
) -> Result<Option<InventoryEventSummary>, AppError> {
    Ok(inventory_events::Entity::find()
        .filter(inventory_events::Column::VariantId.eq(variant_id))
        .order_by_desc(inventory_events::Column::CreatedAt)
        .one(&state.db)
        .await?
        .map(|event| InventoryEventSummary {
            event_type: event.event_type,
            actor: event.actor,
            message: event.message,
            reason: event.reason,
            created_at: event.created_at,
        }))
}

fn build_inventory_event_message(
    previous_stock_quantity: i32,
    previous_is_active: bool,
    next_stock_quantity: i32,
    next_is_active: bool,
    sku: &str,
) -> String {
    let mut changes = Vec::new();

    if previous_stock_quantity != next_stock_quantity {
        changes.push(format!(
            "stock {} -> {}",
            previous_stock_quantity, next_stock_quantity
        ));
    }

    if previous_is_active != next_is_active {
        changes.push(format!(
            "status {} -> {}",
            if previous_is_active { "active" } else { "inactive" },
            if next_is_active { "active" } else { "inactive" }
        ));
    }

    format!("Manual inventory adjustment for {}: {}", sku, changes.join(" · "))
}

async fn build_admin_product_detail(
    state: &AppState,
    product: products::Model,
) -> Result<AdminProductDetail, AppError> {
    let category = match product.category_id {
        Some(category_id) => categories::Entity::find_by_id(category_id)
            .one(&state.db)
            .await?,
        None => None,
    };

    let variants = product_variants::Entity::find()
        .filter(product_variants::Column::ProductId.eq(product.id))
        .order_by_asc(product_variants::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let images = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(product.id))
        .order_by_desc(product_images::Column::IsPrimary)
        .order_by_asc(product_images::Column::SortOrder)
        .all(&state.db)
        .await?;

    let image_assets: Vec<ProductImageAsset> = images
        .into_iter()
        .map(|image| ProductImageAsset {
            image_url: image.image_url,
            alt_text: image.alt_text,
            is_primary: image.is_primary,
            sort_order: image.sort_order,
        })
        .collect();

    let primary_image_url = image_assets
        .iter()
        .find(|image| image.is_primary)
        .or_else(|| image_assets.first())
        .map(|image| image.image_url.clone());

    Ok(AdminProductDetail {
        id: product.id,
        name: product.name,
        slug: product.slug,
        category: category.as_ref().map(|record| record.name.clone()),
        category_slug: category.as_ref().map(|record| record.slug.clone()),
        short_description: product.short_description,
        description: product.description,
        status: product.status,
        is_featured: product.is_featured,
        primary_image_url,
        images: image_assets,
        variants: variants
            .into_iter()
            .map(|variant| ProductVariantOption {
                id: variant.id,
                name: variant.name,
                sku: variant.sku,
                size: variant.size,
                color: variant.color,
                price: variant.price,
                compare_at_price: variant.compare_at_price,
                stock_quantity: variant.stock_quantity,
                is_active: variant.is_active,
            })
            .collect(),
    })
}

async fn sync_product_images<C>(
    db: &C,
    product_id: sea_orm::prelude::Uuid,
    product_name: &str,
    primary_image_url: Option<String>,
    gallery_image_urls: Option<Vec<String>>,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<(), AppError>
where
    C: sea_orm::ConnectionTrait,
{
    let mut image_urls = Vec::new();

    if let Some(primary) = primary_image_url.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        image_urls.push(primary.to_owned());
    }

    for image_url in gallery_image_urls.unwrap_or_default() {
        let image_url = image_url.trim().to_owned();
        if !image_url.is_empty() && !image_urls.contains(&image_url) {
            image_urls.push(image_url);
        }
    }

    product_images::Entity::delete_many()
        .filter(product_images::Column::ProductId.eq(product_id))
        .exec(db)
        .await?;

    for (index, image_url) in image_urls.into_iter().enumerate() {
        product_images::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            product_id: Set(product_id),
            variant_id: Set(None),
            image_url: Set(image_url),
            alt_text: Set(Some(product_name.to_owned())),
            sort_order: Set(index as i32),
            is_primary: Set(index == 0),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;
    }

    Ok(())
}

async fn sync_product_variants<C>(
    db: &C,
    product_id: sea_orm::prelude::Uuid,
    submitted_variants: Vec<CreateProductVariantPayload>,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<(), AppError>
where
    C: sea_orm::ConnectionTrait,
{
    if submitted_variants.is_empty() {
        return Err(AppError::BadRequest("at least one variant is required".to_owned()));
    }

    let existing_variants = product_variants::Entity::find()
        .filter(product_variants::Column::ProductId.eq(product_id))
        .all(db)
        .await?;

    let mut retained_variant_ids = Vec::new();

    for submitted in submitted_variants {
        let trimmed_sku = submitted.sku.trim().to_owned();
        if trimmed_sku.is_empty() {
            return Err(AppError::BadRequest("variant sku is required".to_owned()));
        }

        let variant_name = submitted
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Default Variant")
            .to_owned();

        if let Some(variant_id) = submitted.id {
            let existing = existing_variants
                .iter()
                .find(|variant| variant.id == variant_id)
                .ok_or_else(|| AppError::BadRequest("invalid variant id for product".to_owned()))?;

            let mut active_variant = product_variants::ActiveModel::from(existing.clone());
            active_variant.name = Set(variant_name);
            active_variant.sku = Set(trimmed_sku);
            active_variant.size = Set(normalize_optional_string(submitted.size));
            active_variant.color = Set(normalize_optional_string(submitted.color));
            active_variant.price = Set(parse_decimal(&submitted.price));
            active_variant.compare_at_price = Set(
                submitted.compare_at_price.as_deref().map(parse_decimal),
            );
            active_variant.stock_quantity = Set(submitted.stock_quantity.unwrap_or(0));
            active_variant.is_active = Set(true);
            active_variant.updated_at = Set(now);
            let updated = active_variant.update(db).await?;
            retained_variant_ids.push(updated.id);
        } else {
            let created = product_variants::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                product_id: Set(product_id),
                name: Set(variant_name),
                sku: Set(trimmed_sku),
                size: Set(normalize_optional_string(submitted.size)),
                color: Set(normalize_optional_string(submitted.color)),
                price: Set(parse_decimal(&submitted.price)),
                compare_at_price: Set(submitted.compare_at_price.as_deref().map(parse_decimal)),
                stock_quantity: Set(submitted.stock_quantity.unwrap_or(0)),
                is_active: Set(true),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(db)
            .await?;
            retained_variant_ids.push(created.id);
        }
    }

    for existing in existing_variants {
        if retained_variant_ids.contains(&existing.id) {
            continue;
        }

        let has_orders = order_items::Entity::find()
            .filter(order_items::Column::VariantId.eq(existing.id))
            .one(db)
            .await?
            .is_some();

        if has_orders {
            let mut active_variant = product_variants::ActiveModel::from(existing);
            active_variant.is_active = Set(false);
            active_variant.updated_at = Set(now);
            active_variant.update(db).await?;
        } else {
            product_variants::Entity::delete_by_id(existing.id).exec(db).await?;
        }
    }

    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

async fn upload_product_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<AdminUploadResponse>), AppError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or(AppError::StorageNotConfigured)?;

    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("file") {
            continue;
        }

        let file_name = field
            .file_name()
            .map(sanitize_file_name)
            .unwrap_or_else(|| format!("upload-{}.bin", uuid::Uuid::new_v4()));
        let content_type = field.content_type().map(str::to_owned);
        let bytes = field.bytes().await?.to_vec();
        let object_path = format!(
            "products/uploads/{}-{}",
            Utc::now().format("%Y%m%d%H%M%S"),
            file_name,
        );

        let public_url = storage
            .upload_public_image(&object_path, bytes, content_type)
            .await
            .map_err(|_| AppError::Storage)?;

        return Ok((
            StatusCode::CREATED,
            Json(AdminUploadResponse {
                object_path,
                public_url,
            }),
        ));
    }

    Err(AppError::BadRequest("missing file field".to_owned()))
}

async fn fetch_catalog_products(
    state: &AppState,
    collection: Option<String>,
    search: Option<String>,
    sort: Option<String>,
) -> Result<Vec<CatalogProductCard>, AppError> {
    let mut query = products::Entity::find().filter(products::Column::Status.eq("active"));

    if let Some(search_term) = search.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        query = query.filter(products::Column::Name.contains(search_term));
    }

    match sort.as_deref() {
        Some("name") => {
            query = query.order_by_asc(products::Column::Name);
        }
        _ => {
            query = query.order_by_desc(products::Column::IsFeatured);
            query = query.order_by_desc(products::Column::CreatedAt);
        }
    }

    let records = query.all(&state.db).await?;
    let collection_slug = collection
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "all");

    let mut items = Vec::new();

    for product in records {
        let category = match product.category_id {
            Some(category_id) => categories::Entity::find_by_id(category_id)
                .one(&state.db)
                .await?,
            None => None,
        };

        if let Some(expected_slug) = collection_slug {
            match category.as_ref().map(|record| record.slug.as_str()) {
                Some(actual_slug) if actual_slug == expected_slug => {}
                _ if expected_slug == "sale" || expected_slug == "new-in" => {}
                _ => continue,
            }
        }

        let primary_image = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(product.id))
            .filter(product_images::Column::IsPrimary.eq(true))
            .one(&state.db)
            .await?;

        let default_variant = product_variants::Entity::find()
            .filter(product_variants::Column::ProductId.eq(product.id))
            .filter(product_variants::Column::IsActive.eq(true))
            .order_by_asc(product_variants::Column::CreatedAt)
            .one(&state.db)
            .await?;

        items.push(CatalogProductCard {
            id: product.id,
            slug: product.slug,
            name: product.name,
            category: category.as_ref().map(|record| record.name.clone()),
            category_slug: category.as_ref().map(|record| record.slug.clone()),
            short_description: product.short_description,
            price: default_variant.map(|variant| variant.price),
            primary_image_url: primary_image.map(|image| image.image_url),
        });
    }

    if let Some(sort_kind) = sort.as_deref() {
        match sort_kind {
            "price-low" => items.sort_by(|left, right| left.price.cmp(&right.price)),
            "price-high" => items.sort_by(|left, right| right.price.cmp(&left.price)),
            _ => {}
        }
    }

    Ok(items)
}

fn parse_decimal(value: &str) -> sea_orm::prelude::Decimal {
    value.parse().expect("decimal payload to be valid")
}

fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '-',
        })
        .collect()
}
