use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "inventory_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Uuid,
    pub event_type: String,
    pub actor: String,
    pub message: String,
    pub reason: String,
    pub previous_stock_quantity: Option<i32>,
    pub next_stock_quantity: Option<i32>,
    pub previous_is_active: Option<bool>,
    pub next_is_active: Option<bool>,
    pub created_at: DateTimeWithTimeZone,
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
    #[sea_orm(
        belongs_to = "super::product_variants::Entity",
        from = "Column::VariantId",
        to = "super::product_variants::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Variant,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Variant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
