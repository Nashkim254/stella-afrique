use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "order_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    pub unit_price: Decimal,
    pub quantity: i32,
    pub line_total: Decimal,
    pub image_url: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::orders::Entity",
        from = "Column::OrderId",
        to = "super::orders::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Order,
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "Column::ProductId",
        to = "super::products::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    Product,
    #[sea_orm(
        belongs_to = "super::product_variants::Entity",
        from = "Column::VariantId",
        to = "super::product_variants::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ProductVariant,
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
