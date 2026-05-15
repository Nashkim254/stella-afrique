mod app;
mod config;
mod entities;
mod error;
mod routes;
mod state;

use anyhow::Context;
use app::build_app;
use config::Config;
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env()?;

    let mut options = ConnectOptions::new(config.database_url.clone());
    options
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(60))
        .sqlx_logging(config.app_env != "production");

    let db = Database::connect(options)
        .await
        .context("failed to connect to postgres")?;

    let app = build_app(config.clone(), db);
    let listener = TcpListener::bind(config.bind_addr())
        .await
        .context("failed to bind tcp listener")?;

    info!("backend listening on {}", config.bind_addr());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum server error")
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stellafrique_backend=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        if let Ok(mut signal) = signal(SignalKind::terminate()) {
            signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
