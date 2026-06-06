use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Withdrawals::Table)
                    .if_not_exists()
                    .col(uuid(Withdrawals::Id).primary_key())
                    .col(string(Withdrawals::ReleaseReference).unique_key())
                    .col(decimal(Withdrawals::Amount))
                    .col(string(Withdrawals::Currency))
                    .col(string(Withdrawals::DestinationType))
                    .col(string(Withdrawals::Destination))
                    .col(string(Withdrawals::Status))
                    .col(string_null(Withdrawals::PayoutId))
                    .col(string_null(Withdrawals::ReceiptNumber))
                    .col(string_null(Withdrawals::ExternalRequestId))
                    .col(text_null(Withdrawals::FailureReason))
                    .col(string(Withdrawals::RequestedByEmail))
                    .col(string(Withdrawals::RequestedByRole))
                    .col(
                        timestamp_with_time_zone(Withdrawals::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Withdrawals::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Withdrawals::CompletedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-withdrawals-status")
                    .table(Withdrawals::Table)
                    .col(Withdrawals::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Withdrawals::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Withdrawals {
    Table,
    Id,
    ReleaseReference,
    Amount,
    Currency,
    DestinationType,
    Destination,
    Status,
    PayoutId,
    ReceiptNumber,
    ExternalRequestId,
    FailureReason,
    RequestedByEmail,
    RequestedByRole,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
}
