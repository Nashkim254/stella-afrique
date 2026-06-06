use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .col(string(Users::Email).unique_key())
                    .col(string(Users::FullName))
                    .col(text(Users::PasswordHash))
                    .col(text(Users::PasswordSalt))
                    .col(boolean(Users::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(Users::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Users::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-users-email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    FullName,
    PasswordHash,
    PasswordSalt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
