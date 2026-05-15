use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product_images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub public_id: String,
    pub url: String,
    pub alt_text: Option<String>,
    pub is_primary: bool,
    pub sort_order: i32,
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
