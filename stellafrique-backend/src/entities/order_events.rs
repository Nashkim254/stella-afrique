use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "order_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub order_id: Uuid,
    pub event_type: String,
    pub actor: String,
    pub message: String,
    pub details: Option<String>,
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
    Orders,
}

impl Related<super::orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
