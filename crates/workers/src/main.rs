use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod sources;
mod ingestion;

use sources::{polymarket, metaculus, manifold};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "forecise_workers=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = forecise_shared::Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Workers connected to database");

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Forecise/0.1.0")
        .build()?;

    // Run all workers concurrently
    let pool_clone1 = pool.clone();
    let pool_clone2 = pool.clone();
    let pool_clone3 = pool.clone();
    let client1 = http_client.clone();
    let client2 = http_client.clone();
    let client3 = http_client.clone();

    tracing::info!("Starting data ingestion workers...");

    tokio::select! {
        r = polymarket::run_worker(pool_clone1, client1) => {
            tracing::error!("Polymarket worker exited: {:?}", r);
        }
        r = metaculus::run_worker(pool_clone2, client2) => {
            tracing::error!("Metaculus worker exited: {:?}", r);
        }
        r = manifold::run_worker(pool_clone3, client3) => {
            tracing::error!("Manifold worker exited: {:?}", r);
        }
    }

    Ok(())
}
