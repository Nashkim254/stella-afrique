use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OrderEvents::Table)
                    .if_not_exists()
                    .col(uuid(OrderEvents::Id).primary_key())
                    .col(uuid(OrderEvents::OrderId))
                    .col(string(OrderEvents::EventType))
                    .col(string(OrderEvents::Actor))
                    .col(text(OrderEvents::Message))
                    .col(text_null(OrderEvents::Details))
                    .col(
                        timestamp_with_time_zone(OrderEvents::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_events-order_id")
                            .from(OrderEvents::Table, OrderEvents::OrderId)
                            .to(Orders::Table, Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-order_events-order_id-created_at")
                    .table(OrderEvents::Table)
                    .col(OrderEvents::OrderId)
                    .col(OrderEvents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OrderEvents {
    Table,
    Id,
    OrderId,
    EventType,
    Actor,
    Message,
    Details,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
}
