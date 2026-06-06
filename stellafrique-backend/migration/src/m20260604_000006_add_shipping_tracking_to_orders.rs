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
                    .add_column_if_not_exists(string_null(Orders::ShippingCourier))
                    .add_column_if_not_exists(string_null(Orders::TrackingNumber))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::ShippingCourier)
                    .drop_column(Orders::TrackingNumber)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    ShippingCourier,
    TrackingNumber,
}
