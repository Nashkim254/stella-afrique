use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product_variants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub sku: String,
    pub size: Option<String>,
    pub color: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub stock_quantity: i32,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "Column::ProductId",
        to = "super::products::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Product,
    #[sea_orm(has_many = "super::product_images::Entity")]
    ProductImages,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::product_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductImages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
