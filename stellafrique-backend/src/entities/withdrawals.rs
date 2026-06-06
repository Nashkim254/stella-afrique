use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "withdrawals")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub release_reference: String,
    pub amount: Decimal,
    pub currency: String,
    pub destination_type: String,
    pub destination: String,
    pub status: String,
    pub payout_id: Option<String>,
    pub receipt_number: Option<String>,
    pub external_request_id: Option<String>,
    pub failure_reason: Option<String>,
    pub requested_by_email: String,
    pub requested_by_role: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub completed_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
