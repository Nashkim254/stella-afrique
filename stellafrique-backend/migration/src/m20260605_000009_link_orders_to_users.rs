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
                    .add_column_if_not_exists(uuid_null(Orders::UserId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-orders-user_id")
                            .from_tbl(Orders::Table)
                            .from_col(Orders::UserId)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-orders-user_id")
                    .table(Orders::Table)
                    .col(Orders::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-orders-user_id")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_foreign_key(Alias::new("fk-orders-user_id"))
                    .drop_column(Orders::UserId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
