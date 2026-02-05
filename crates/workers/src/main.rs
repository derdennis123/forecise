use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod sources;
mod ingestion;
mod movement;
mod consensus_worker;
mod briefing;

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

    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let pool4 = pool.clone();
    let pool5 = pool.clone();
    let pool6 = pool.clone();
    let client1 = http_client.clone();
    let client2 = http_client.clone();
    let client3 = http_client.clone();

    tracing::info!("Starting data ingestion workers...");

    tokio::select! {
        r = polymarket::run_worker(pool1, client1) => {
            tracing::error!("Polymarket worker exited: {:?}", r);
        }
        r = metaculus::run_worker(pool2, client2) => {
            tracing::error!("Metaculus worker exited: {:?}", r);
        }
        r = manifold::run_worker(pool3, client3) => {
            tracing::error!("Manifold worker exited: {:?}", r);
        }
        r = run_movement_detector(pool4) => {
            tracing::error!("Movement detector exited: {:?}", r);
        }
        r = consensus_worker::run_consensus_worker(pool5) => {
            tracing::error!("Consensus worker exited: {:?}", r);
        }
        r = briefing::run_briefing_generator(pool6) => {
            tracing::error!("Briefing generator exited: {:?}", r);
        }
    }

    Ok(())
}

async fn run_movement_detector(pool: sqlx::PgPool) -> Result<()> {
    // Wait for initial data ingestion
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    loop {
        match movement::detect_movements(&pool).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Detected {} significant movements", count);
                }
            }
            Err(e) => tracing::error!("Movement detection error: {}", e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
    }
}
