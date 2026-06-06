use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StaffUsers::Table)
                    .if_not_exists()
                    .col(uuid(StaffUsers::Id).primary_key())
                    .col(string(StaffUsers::Email).unique_key())
                    .col(string(StaffUsers::FullName))
                    .col(string(StaffUsers::Role))
                    .col(text(StaffUsers::PasswordHash))
                    .col(text(StaffUsers::PasswordSalt))
                    .col(boolean(StaffUsers::IsActive).default(true))
                    .col(timestamp_with_time_zone_null(StaffUsers::LastLoginAt))
                    .col(
                        timestamp_with_time_zone(StaffUsers::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(StaffUsers::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-staff-users-email")
                    .table(StaffUsers::Table)
                    .col(StaffUsers::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-staff-users-role")
                    .table(StaffUsers::Table)
                    .col(StaffUsers::Role)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StaffUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum StaffUsers {
    Table,
    Id,
    Email,
    FullName,
    Role,
    PasswordHash,
    PasswordSalt,
    IsActive,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
}
