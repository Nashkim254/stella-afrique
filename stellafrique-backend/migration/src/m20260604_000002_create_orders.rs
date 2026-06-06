use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(uuid(Orders::Id).primary_key())
                    .col(string(Orders::OrderNumber).unique_key())
                    .col(string(Orders::Status))
                    .col(string(Orders::Currency))
                    .col(decimal(Orders::SubtotalAmount))
                    .col(decimal(Orders::TotalAmount))
                    .col(string(Orders::CustomerName))
                    .col(string(Orders::CustomerEmail))
                    .col(string(Orders::CustomerPhone))
                    .col(string(Orders::ShippingAddressLine1))
                    .col(string_null(Orders::ShippingAddressLine2))
                    .col(string(Orders::ShippingCity))
                    .col(string_null(Orders::ShippingRegion))
                    .col(string_null(Orders::ShippingPostalCode))
                    .col(string(Orders::ShippingCountry))
                    .col(text_null(Orders::Notes))
                    .col(
                        timestamp_with_time_zone(Orders::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Orders::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrderItems::Table)
                    .if_not_exists()
                    .col(uuid(OrderItems::Id).primary_key())
                    .col(uuid(OrderItems::OrderId))
                    .col(uuid(OrderItems::ProductId))
                    .col(uuid_null(OrderItems::VariantId))
                    .col(string(OrderItems::ProductName))
                    .col(string_null(OrderItems::VariantName))
                    .col(string_null(OrderItems::Sku))
                    .col(string_null(OrderItems::Size))
                    .col(string_null(OrderItems::Color))
                    .col(decimal(OrderItems::UnitPrice))
                    .col(integer(OrderItems::Quantity))
                    .col(decimal(OrderItems::LineTotal))
                    .col(text_null(OrderItems::ImageUrl))
                    .col(
                        timestamp_with_time_zone(OrderItems::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_items-order_id")
                            .from(OrderItems::Table, OrderItems::OrderId)
                            .to(Orders::Table, Orders::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_items-product_id")
                            .from(OrderItems::Table, OrderItems::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-order_items-variant_id")
                            .from(OrderItems::Table, OrderItems::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-orders-order_number")
                    .table(Orders::Table)
                    .col(Orders::OrderNumber)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-orders-customer_email")
                    .table(Orders::Table)
                    .col(Orders::CustomerEmail)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-order_items-order_id")
                    .table(OrderItems::Table)
                    .col(OrderItems::OrderId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OrderItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Id,
    OrderNumber,
    Status,
    Currency,
    SubtotalAmount,
    TotalAmount,
    CustomerName,
    CustomerEmail,
    CustomerPhone,
    ShippingAddressLine1,
    ShippingAddressLine2,
    ShippingCity,
    ShippingRegion,
    ShippingPostalCode,
    ShippingCountry,
    Notes,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OrderItems {
    Table,
    Id,
    OrderId,
    ProductId,
    VariantId,
    ProductName,
    VariantName,
    Sku,
    Size,
    Color,
    UnitPrice,
    Quantity,
    LineTotal,
    ImageUrl,
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
