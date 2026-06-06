use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(string_null(Orders::PaymentMethod))
                    .add_column_if_not_exists(string_null(Orders::PaymentReference))
                    .add_column_if_not_exists(timestamp_with_time_zone_null(Orders::PaidAt))
                    .add_column_if_not_exists(timestamp_with_time_zone_null(Orders::FulfilledAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::PaymentMethod)
                    .drop_column(Orders::PaymentReference)
                    .drop_column(Orders::PaidAt)
                    .drop_column(Orders::FulfilledAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    PaymentMethod,
    PaymentReference,
    PaidAt,
    FulfilledAt,
}
