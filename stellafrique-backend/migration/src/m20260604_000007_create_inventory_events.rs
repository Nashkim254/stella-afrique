use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InventoryEvents::Table)
                    .if_not_exists()
                    .col(uuid(InventoryEvents::Id).primary_key())
                    .col(uuid(InventoryEvents::ProductId))
                    .col(uuid(InventoryEvents::VariantId))
                    .col(string(InventoryEvents::EventType))
                    .col(string(InventoryEvents::Actor))
                    .col(text(InventoryEvents::Message))
                    .col(text(InventoryEvents::Reason))
                    .col(integer_null(InventoryEvents::PreviousStockQuantity))
                    .col(integer_null(InventoryEvents::NextStockQuantity))
                    .col(boolean_null(InventoryEvents::PreviousIsActive))
                    .col(boolean_null(InventoryEvents::NextIsActive))
                    .col(
                        timestamp_with_time_zone(InventoryEvents::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-inventory_events-product_id")
                            .from(InventoryEvents::Table, InventoryEvents::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-inventory_events-variant_id")
                            .from(InventoryEvents::Table, InventoryEvents::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-inventory_events-variant_id-created_at")
                    .table(InventoryEvents::Table)
                    .col(InventoryEvents::VariantId)
                    .col(InventoryEvents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InventoryEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InventoryEvents {
    Table,
    Id,
    ProductId,
    VariantId,
    EventType,
    Actor,
    Message,
    Reason,
    PreviousStockQuantity,
    NextStockQuantity,
    PreviousIsActive,
    NextIsActive,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ProductVariants {
    Table,
    Id,
}
