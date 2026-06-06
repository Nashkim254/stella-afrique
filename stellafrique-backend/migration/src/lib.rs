pub use sea_orm_migration::prelude::*;

mod m20250515_000001_create_catalog;
mod m20260604_000002_create_orders;
mod m20260604_000003_add_payment_status_to_orders;
mod m20260604_000004_add_order_timestamps_and_payment_metadata;
mod m20260604_000005_create_order_events;
mod m20260604_000006_add_shipping_tracking_to_orders;
mod m20260604_000007_create_inventory_events;
mod m20260604_000008_create_users;
mod m20260605_000009_link_orders_to_users;
mod m20260605_000010_add_profile_fields_to_users;
mod m20260605_000011_create_staff_users;
mod m20260606_000012_create_withdrawals;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250515_000001_create_catalog::Migration),
            Box::new(m20260604_000002_create_orders::Migration),
            Box::new(m20260604_000003_add_payment_status_to_orders::Migration),
            Box::new(m20260604_000004_add_order_timestamps_and_payment_metadata::Migration),
            Box::new(m20260604_000005_create_order_events::Migration),
            Box::new(m20260604_000006_add_shipping_tracking_to_orders::Migration),
            Box::new(m20260604_000007_create_inventory_events::Migration),
            Box::new(m20260604_000008_create_users::Migration),
            Box::new(m20260605_000009_link_orders_to_users::Migration),
            Box::new(m20260605_000010_add_profile_fields_to_users::Migration),
            Box::new(m20260605_000011_create_staff_users::Migration),
            Box::new(m20260606_000012_create_withdrawals::Migration),
        ]
    }
}
