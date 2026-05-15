use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::products::Entity")]
    Products,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
