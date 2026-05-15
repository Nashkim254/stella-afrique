use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub care_instructions: Option<String>,
    pub brand: Option<String>,
    pub is_active: bool,
    pub is_featured: bool,
    pub avg_rating: Decimal,
    pub review_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::CategoryId",
        to = "super::categories::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Categories,
    #[sea_orm(has_many = "super::product_variants::Entity")]
    ProductVariants,
    #[sea_orm(has_many = "super::product_images::Entity")]
    ProductImages,
}

impl Related<super::categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Categories.def()
    }
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariants.def()
    }
}

impl Related<super::product_images::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductImages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
