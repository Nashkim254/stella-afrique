use anyhow::Context;
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use stellafrique_backend::{config::Config, store_seed::reseed_catalog};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let mut options = ConnectOptions::new(config.database_url.clone());
    options
        .max_connections(10)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30))
        .sqlx_logging(config.app_env != "production");

    let db = Database::connect(options)
        .await
        .context("failed to connect to postgres for seeding")?;

    reseed_catalog(&db).await.context("failed to seed catalog")?;

    println!("Catalog seed complete.");

    Ok(())
}
