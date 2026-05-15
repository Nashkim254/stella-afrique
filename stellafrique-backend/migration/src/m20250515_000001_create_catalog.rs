use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(uuid(Categories::Id).primary_key())
                    .col(string(Categories::Name))
                    .col(string(Categories::Slug).unique_key())
                    .col(text_null(Categories::Description))
                    .col(text_null(Categories::ImageUrl))
                    .col(boolean(Categories::IsActive).default(true))
                    .col(integer(Categories::SortOrder).default(0))
                    .col(
                        timestamp_with_time_zone(Categories::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Categories::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(uuid(Products::Id).primary_key())
                    .col(uuid_null(Products::CategoryId))
                    .col(string(Products::Name))
                    .col(string(Products::Slug).unique_key())
                    .col(text_null(Products::ShortDescription))
                    .col(text_null(Products::Description))
                    .col(string(Products::Status))
                    .col(boolean(Products::IsFeatured).default(false))
                    .col(
                        timestamp_with_time_zone(Products::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Products::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-products-category_id")
                            .from(Products::Table, Products::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductVariants::Table)
                    .if_not_exists()
                    .col(uuid(ProductVariants::Id).primary_key())
                    .col(uuid(ProductVariants::ProductId))
                    .col(string(ProductVariants::Name))
                    .col(string(ProductVariants::Sku).unique_key())
                    .col(string_null(ProductVariants::Size))
                    .col(string_null(ProductVariants::Color))
                    .col(decimal(ProductVariants::Price, 12, 2))
                    .col(decimal_null(ProductVariants::CompareAtPrice, 12, 2))
                    .col(integer(ProductVariants::StockQuantity).default(0))
                    .col(boolean(ProductVariants::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(ProductVariants::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProductVariants::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product_variants-product_id")
                            .from(ProductVariants::Table, ProductVariants::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductImages::Table)
                    .if_not_exists()
                    .col(uuid(ProductImages::Id).primary_key())
                    .col(uuid(ProductImages::ProductId))
                    .col(uuid_null(ProductImages::VariantId))
                    .col(text(ProductImages::ImageUrl))
                    .col(text_null(ProductImages::AltText))
                    .col(integer(ProductImages::SortOrder).default(0))
                    .col(boolean(ProductImages::IsPrimary).default(false))
                    .col(
                        timestamp_with_time_zone(ProductImages::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProductImages::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product_images-product_id")
                            .from(ProductImages::Table, ProductImages::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-product_images-variant_id")
                            .from(ProductImages::Table, ProductImages::VariantId)
                            .to(ProductVariants::Table, ProductVariants::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-categories-slug")
                    .table(Categories::Table)
                    .col(Categories::Slug)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-products-category_id")
                    .table(Products::Table)
                    .col(Products::CategoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-products-status")
                    .table(Products::Table)
                    .col(Products::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-product_variants-product_id")
                    .table(ProductVariants::Table)
                    .col(ProductVariants::ProductId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-product_images-product_id")
                    .table(ProductImages::Table)
                    .col(ProductImages::ProductId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductImages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProductVariants::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    Slug,
    Description,
    ImageUrl,
    IsActive,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    CategoryId,
    Name,
    Slug,
    ShortDescription,
    Description,
    Status,
    IsFeatured,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProductVariants {
    Table,
    Id,
    ProductId,
    Name,
    Sku,
    Size,
    Color,
    Price,
    CompareAtPrice,
    StockQuantity,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ProductImages {
    Table,
    Id,
    ProductId,
    VariantId,
    ImageUrl,
    AltText,
    SortOrder,
    IsPrimary,
    CreatedAt,
    UpdatedAt,
}

fn uuid<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).uuid().not_null().take()
}

fn uuid_null<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).uuid().null().take()
}

fn string<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).string().not_null().take()
}

fn string_null<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).string().null().take()
}

fn text<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).text().not_null().take()
}

fn text_null<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).text().null().take()
}

fn integer<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).integer().not_null().take()
}

fn boolean<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column).boolean().not_null().take()
}

fn decimal<T>(column: T, precision: u32, scale: u32) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column)
        .decimal_len(precision, scale)
        .not_null()
        .take()
}

fn decimal_null<T>(column: T, precision: u32, scale: u32) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column)
        .decimal_len(precision, scale)
        .null()
        .take()
}

fn timestamp_with_time_zone<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(column)
        .timestamp_with_time_zone()
        .not_null()
        .take()
}
