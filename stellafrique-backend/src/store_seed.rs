use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use sea_orm::prelude::{Decimal, Uuid};

use crate::entities::{categories, product_images, product_variants, products};

type SeedResult<T> = Result<T, sea_orm::DbErr>;

pub struct SeedCategory {
    pub id: Uuid,
    pub name: &'static str,
    pub slug: &'static str,
    pub description: &'static str,
    pub image_url: &'static str,
    pub sort_order: i32,
}

pub struct SeedProduct {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: &'static str,
    pub slug: &'static str,
    pub short_description: &'static str,
    pub description: &'static str,
    pub is_featured: bool,
    pub images: Vec<&'static str>,
    pub variants: Vec<SeedVariant>,
}

pub struct SeedVariant {
    pub id: Uuid,
    pub name: &'static str,
    pub sku: &'static str,
    pub size: &'static str,
    pub color: &'static str,
    pub price: &'static str,
    pub compare_at_price: Option<&'static str>,
    pub stock_quantity: i32,
}

pub async fn reseed_catalog(db: &DatabaseConnection) -> SeedResult<()> {
    product_images::Entity::delete_many().exec(db).await?;
    product_variants::Entity::delete_many().exec(db).await?;
    products::Entity::delete_many().exec(db).await?;
    categories::Entity::delete_many().exec(db).await?;

    let now = chrono::Utc::now().fixed_offset();

    for category in seed_categories() {
        categories::ActiveModel {
            id: Set(category.id),
            name: Set(category.name.to_owned()),
            slug: Set(category.slug.to_owned()),
            description: Set(Some(category.description.to_owned())),
            image_url: Set(Some(category.image_url.to_owned())),
            is_active: Set(true),
            sort_order: Set(category.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;
    }

    for product in seed_products() {
        products::ActiveModel {
            id: Set(product.id),
            category_id: Set(Some(product.category_id)),
            name: Set(product.name.to_owned()),
            slug: Set(product.slug.to_owned()),
            short_description: Set(Some(product.short_description.to_owned())),
            description: Set(Some(product.description.to_owned())),
            status: Set("active".to_owned()),
            is_featured: Set(product.is_featured),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;

        for (index, image_url) in product.images.iter().enumerate() {
            product_images::ActiveModel {
                id: Set(uuid_for(&format!("{}-image-{}", product.slug, index + 1))),
                product_id: Set(product.id),
                variant_id: Set(None),
                image_url: Set((*image_url).to_owned()),
                alt_text: Set(Some(product.name.to_owned())),
                sort_order: Set(index as i32),
                is_primary: Set(index == 0),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(db)
            .await?;
        }

        for variant in product.variants {
            product_variants::ActiveModel {
                id: Set(variant.id),
                product_id: Set(product.id),
                name: Set(variant.name.to_owned()),
                sku: Set(variant.sku.to_owned()),
                size: Set(Some(variant.size.to_owned())),
                color: Set(Some(variant.color.to_owned())),
                price: Set(decimal(variant.price)),
                compare_at_price: Set(variant.compare_at_price.map(decimal)),
                stock_quantity: Set(variant.stock_quantity),
                is_active: Set(true),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(db)
            .await?;
        }
    }

    Ok(())
}

fn seed_categories() -> Vec<SeedCategory> {
    vec![
        SeedCategory {
            id: uuid_for("cat-knitwear"),
            name: "Knitwear",
            slug: "knitwear",
            description: "Soft layers, cardigans, and elevated knit staples.",
            image_url: "/images/products/fashion-06.jpg",
            sort_order: 1,
        },
        SeedCategory {
            id: uuid_for("cat-dresses"),
            name: "Dresses",
            slug: "dresses",
            description: "Easy occasion dresses and structured day shapes.",
            image_url: "/images/products/fashion-05.jpg",
            sort_order: 2,
        },
        SeedCategory {
            id: uuid_for("cat-accessories"),
            name: "Accessories",
            slug: "accessories",
            description: "Giftable finishing pieces and wardrobe accents.",
            image_url: "/images/products/fashion-16.jpg",
            sort_order: 3,
        },
        SeedCategory {
            id: uuid_for("cat-outerwear"),
            name: "Outerwear",
            slug: "outerwear",
            description: "Blazers and layers for polished everyday styling.",
            image_url: "/images/products/fashion-15.jpg",
            sort_order: 4,
        },
        SeedCategory {
            id: uuid_for("cat-tops"),
            name: "Tops",
            slug: "tops",
            description: "Blouses and styling pieces to anchor the edit.",
            image_url: "/images/products/fashion-09.jpg",
            sort_order: 5,
        },
        SeedCategory {
            id: uuid_for("cat-shirts"),
            name: "Shirts",
            slug: "shirts",
            description: "Clean shirting for workdays and soft layering.",
            image_url: "/images/products/fashion-10.jpg",
            sort_order: 6,
        },
        SeedCategory {
            id: uuid_for("cat-basics"),
            name: "Basics",
            slug: "basics",
            description: "Simple essentials for everyday wardrobes.",
            image_url: "/images/products/fashion-12.jpg",
            sort_order: 7,
        },
        SeedCategory {
            id: uuid_for("cat-matching-set"),
            name: "Matching Set",
            slug: "matching-set",
            description: "Sets and coordinated looks with minimal effort.",
            image_url: "/images/products/fashion-03.jpg",
            sort_order: 8,
        },
    ]
}

fn seed_products() -> Vec<SeedProduct> {
    vec![
        product(
            "tailored-linen-set",
            "cat-matching-set",
            "Tailored Linen Set",
            "Lightweight two-piece look built for warm city days.",
            "A clean matching set cut for movement, warm afternoons, and polished weekday dressing.",
            true,
            vec!["/images/products/fashion-01.jpg", "/images/products/fashion-10.jpg", "/images/products/fashion-15.jpg"],
            vec![variant("tailored-linen-set-sand-m", "Set M", "STL-LIN-SET-M", "M", "Sand", "5400", Some("6200"), 8)],
        ),
        product(
            "structured-weekend-dress",
            "cat-dresses",
            "Structured Weekend Dress",
            "Clean silhouette with a soft drape and practical pockets.",
            "A soft occasion dress with enough structure to carry from lunch dates into evening plans.",
            true,
            vec!["/images/products/fashion-02.jpg", "/images/products/fashion-18.jpg", "/images/products/fashion-20.jpg"],
            vec![variant("structured-weekend-dress-rose-m", "Dress M", "STL-DRS-WKD-M", "M", "Rose", "6200", Some("7200"), 10)],
        ),
        product(
            "soft-neutral-knit",
            "cat-knitwear",
            "Soft Neutral Knit",
            "Layer-friendly knit with an easy relaxed fit.",
            "The everyday knit layer that works with denim, tailoring, and travel packing.",
            true,
            vec!["/images/products/fashion-06.jpg", "/images/products/fashion-12.jpg", "/images/products/fashion-19.jpg"],
            vec![variant("soft-neutral-knit-oat-m", "Knit M", "STL-KNT-SFT-M", "M", "Oat", "4600", None, 12)],
        ),
        product(
            "everyday-occasion-blazer",
            "cat-outerwear",
            "Everyday Occasion Blazer",
            "Clean tailoring for work, dinner, and weekends.",
            "Relaxed tailoring with enough shape for work looks, celebrations, and smart gifting.",
            true,
            vec!["/images/products/fashion-15.jpg", "/images/products/fashion-11.jpg", "/images/products/fashion-16.jpg"],
            vec![variant("everyday-occasion-blazer-cream-l", "Blazer L", "STL-BLZ-EVG-L", "L", "Cream", "7800", Some("8600"), 6)],
        ),
        product(
            "cloudline-cardigan",
            "cat-knitwear",
            "Cloudline Cardigan",
            "A soft cardigan made for layering over dresses and simple tanks without adding bulk.",
            "A soft cardigan made for layering over dresses and simple tanks without adding bulk.",
            false,
            vec!["/images/products/fashion-17.jpg", "/images/products/fashion-12.jpg", "/images/products/fashion-06.jpg"],
            vec![variant("cloudline-cardigan-blush-m", "Cardigan M", "STL-KNT-CLD-M", "M", "Blush", "4900", None, 15)],
        ),
        product(
            "rose-edit-blouse",
            "cat-tops",
            "Rose Edit Blouse",
            "A feminine blouse with enough structure to keep tailoring, skirts, and denim feeling intentional.",
            "A feminine blouse with enough structure to keep tailoring, skirts, and denim feeling intentional.",
            false,
            vec!["/images/products/fashion-09.jpg", "/images/products/fashion-13.jpg", "/images/products/fashion-20.jpg"],
            vec![variant("rose-edit-blouse-rose-s", "Blouse S", "STL-TOP-RSE-S", "S", "Rose", "4300", None, 11)],
        ),
        product(
            "soft-studio-layer",
            "cat-outerwear",
            "Soft Studio Layer",
            "An easy outer layer designed for neat, minimal outfits and polished everyday errands.",
            "An easy outer layer designed for neat, minimal outfits and polished everyday errands.",
            false,
            vec!["/images/products/fashion-07.jpg", "/images/products/fashion-18.jpg", "/images/products/fashion-02.jpg"],
            vec![variant("soft-studio-layer-stone-l", "Layer L", "STL-OUT-STU-L", "L", "Stone", "6900", Some("7600"), 7)],
        ),
        product(
            "weekend-shift-dress",
            "cat-dresses",
            "Weekend Shift Dress",
            "A simple dress shape that feels dressed without needing much styling effort.",
            "A simple dress shape that feels dressed without needing much styling effort.",
            false,
            vec!["/images/products/fashion-18.jpg", "/images/products/fashion-05.jpg", "/images/products/fashion-03.jpg"],
            vec![variant("weekend-shift-dress-ivory-m", "Dress M", "STL-DRS-SHF-M", "M", "Ivory", "6200", None, 9)],
        ),
        product(
            "minimal-day-shirt",
            "cat-shirts",
            "Minimal Day Shirt",
            "A clean shirt for office mornings, weekend markets, and soft layering under knitwear.",
            "A clean shirt for office mornings, weekend markets, and soft layering under knitwear.",
            false,
            vec!["/images/products/fashion-10.jpg", "/images/products/fashion-07.jpg", "/images/products/fashion-01.jpg"],
            vec![variant("minimal-day-shirt-white-m", "Shirt M", "STL-SHR-MIN-M", "M", "White", "4500", None, 13)],
        ),
        product(
            "signature-lounge-knit",
            "cat-basics",
            "Signature Lounge Knit",
            "The off-duty knit set that keeps comfort high without losing shape or softness.",
            "The off-duty knit set that keeps comfort high without losing shape or softness.",
            false,
            vec!["/images/products/fashion-12.jpg", "/images/products/fashion-19.jpg", "/images/products/fashion-17.jpg"],
            vec![variant("signature-lounge-knit-cream-s", "Knit S", "STL-BSC-LNG-S", "S", "Cream", "3800", None, 20)],
        ),
        product(
            "pastel-weekend-set",
            "cat-matching-set",
            "Pastel Weekend Set",
            "A playful set for weekend travel, gifting moments, and easy daytime styling.",
            "A playful set for weekend travel, gifting moments, and easy daytime styling.",
            false,
            vec!["/images/products/fashion-03.jpg", "/images/products/fashion-18.jpg", "/images/products/fashion-13.jpg"],
            vec![variant("pastel-weekend-set-pink-m", "Set M", "STL-SET-PST-M", "M", "Pink", "7100", Some("7900"), 5)],
        ),
        product(
            "soft-form-blazer",
            "cat-outerwear",
            "Soft Form Blazer",
            "Relaxed tailoring with enough drape to work over dresses, denim, and knit separates.",
            "Relaxed tailoring with enough drape to work over dresses, denim, and knit separates.",
            false,
            vec!["/images/products/fashion-04.jpg", "/images/products/fashion-15.jpg", "/images/products/fashion-16.jpg"],
            vec![variant("soft-form-blazer-bone-l", "Blazer L", "STL-BLZ-SFT-L", "L", "Bone", "8200", Some("9000"), 4)],
        ),
        product(
            "daylight-knit",
            "cat-knitwear",
            "Daylight Knit",
            "A softer knit option made for layering through cool mornings and rainy afternoons.",
            "A softer knit option made for layering through cool mornings and rainy afternoons.",
            false,
            vec!["/images/products/fashion-19.jpg", "/images/products/fashion-06.jpg", "/images/products/fashion-12.jpg"],
            vec![variant("daylight-knit-oat-m", "Knit M", "STL-KNT-DAY-M", "M", "Oat", "4700", None, 14)],
        ),
        product(
            "rose-layer-shirt",
            "cat-tops",
            "Rose Layer Shirt",
            "A light shirt with a subtle statement colour for brightening neutral wardrobes.",
            "A light shirt with a subtle statement colour for brightening neutral wardrobes.",
            false,
            vec!["/images/products/fashion-20.jpg", "/images/products/fashion-09.jpg", "/images/products/fashion-10.jpg"],
            vec![variant("rose-layer-shirt-clay-s", "Shirt S", "STL-TOP-LYR-S", "S", "Clay", "3900", None, 16)],
        ),
    ]
}

fn product(
    slug: &'static str,
    category_key: &'static str,
    name: &'static str,
    short_description: &'static str,
    description: &'static str,
    is_featured: bool,
    images: Vec<&'static str>,
    variants: Vec<SeedVariant>,
) -> SeedProduct {
    SeedProduct {
        id: uuid_for(&format!("product-{slug}")),
        category_id: uuid_for(category_key),
        name,
        slug,
        short_description,
        description,
        is_featured,
        images,
        variants,
    }
}

fn variant(
    key: &'static str,
    name: &'static str,
    sku: &'static str,
    size: &'static str,
    color: &'static str,
    price: &'static str,
    compare_at_price: Option<&'static str>,
    stock_quantity: i32,
) -> SeedVariant {
    SeedVariant {
        id: uuid_for(key),
        name,
        sku,
        size,
        color,
        price,
        compare_at_price,
        stock_quantity,
    }
}

fn uuid_for(key: &str) -> Uuid {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, key.as_bytes())
}

fn decimal(value: &str) -> Decimal {
    value.parse().expect("seed decimal to be valid")
}
