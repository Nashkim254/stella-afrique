use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(string_null(Users::Phone))
                    .add_column_if_not_exists(string_null(Users::AddressLine1))
                    .add_column_if_not_exists(string_null(Users::AddressLine2))
                    .add_column_if_not_exists(string_null(Users::City))
                    .add_column_if_not_exists(string_null(Users::Region))
                    .add_column_if_not_exists(string_null(Users::PostalCode))
                    .add_column_if_not_exists(string_null(Users::Country))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Country)
                    .drop_column(Users::PostalCode)
                    .drop_column(Users::Region)
                    .drop_column(Users::City)
                    .drop_column(Users::AddressLine2)
                    .drop_column(Users::AddressLine1)
                    .drop_column(Users::Phone)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Phone,
    AddressLine1,
    AddressLine2,
    City,
    Region,
    PostalCode,
    Country,
}
