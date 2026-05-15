use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product_variants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub stock_qty: i32,
    pub low_stock_threshold: i32,
    pub sku: String,
    pub is_active: bool,
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
    Products,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
