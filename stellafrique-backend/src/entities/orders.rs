use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub currency: String,
    pub subtotal_amount: Decimal,
    pub total_amount: Decimal,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub shipping_address_line1: String,
    pub shipping_address_line2: Option<String>,
    pub shipping_city: String,
    pub shipping_region: Option<String>,
    pub shipping_postal_code: Option<String>,
    pub shipping_country: String,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub shipping_courier: Option<String>,
    pub tracking_number: Option<String>,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub fulfilled_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    User,
    #[sea_orm(has_many = "super::order_items::Entity")]
    OrderItems,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::order_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderItems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
