use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn, error};

use crate::ingestion;

const MANIFOLD_API: &str = "https://api.manifold.markets/v0";
const POLL_INTERVAL_SECS: u64 = 600; // 10 minutes

#[derive(Debug, Deserialize)]
struct ManifoldMarket {
    id: String,
    question: Option<String>,
    url: Option<String>,
    probability: Option<f64>,
    volume: Option<f64>,
    #[serde(rename = "totalLiquidity")]
    total_liquidity: Option<f64>,
    #[serde(rename = "isResolved")]
    is_resolved: Option<bool>,
    #[serde(rename = "outcomeType")]
    outcome_type: Option<String>,
    slug: Option<String>,
}

pub async fn run_worker(pool: PgPool, client: Client) -> Result<()> {
    info!("Starting Manifold worker");

    // Stagger start
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    loop {
        match fetch_and_store(&pool, &client).await {
            Ok(count) => info!("Manifold: ingested {} markets", count),
            Err(e) => error!("Manifold worker error: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn fetch_and_store(pool: &PgPool, client: &Client) -> Result<usize> {
    let mut total = 0;

    // Fetch trending markets
    let url = format!("{}/search-markets?term=&sort=liquidity&limit=100&filter=open", MANIFOLD_API);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        warn!("Manifold API returned {}", response.status());
        return Ok(0);
    }

    let markets: Vec<ManifoldMarket> = response.json().await?;

    for market in &markets {
        if let Err(e) = process_market(pool, market).await {
            warn!("Failed to process Manifold market: {}", e);
        } else {
            total += 1;
        }
    }

    Ok(total)
}

async fn process_market(pool: &PgPool, market: &ManifoldMarket) -> Result<()> {
    // Only binary markets
    if market.outcome_type.as_deref() != Some("BINARY") {
        return Ok(());
    }

    if market.is_resolved == Some(true) {
        return Ok(());
    }

    let question = match &market.question {
        Some(q) => q,
        None => return Ok(()),
    };

    let probability = market.probability.unwrap_or(0.5);

    let external_url = market.url.as_deref()
        .or(market.slug.as_ref().map(|s| s.as_str()))
        .map(|u| {
            if u.starts_with("http") { u.to_string() }
            else { format!("https://manifold.markets/{}", u) }
        });

    let metadata = serde_json::json!({
        "total_liquidity": market.total_liquidity,
        "outcome_type": market.outcome_type,
    });

    let source_market_id = ingestion::upsert_source_market(
        pool,
        "manifold",
        &market.id,
        question,
        probability,
        market.volume,
        external_url.as_deref(),
        metadata,
    ).await?;

    let slug = format!("mf-{}", slug_from_title(question));

    ingestion::ensure_unified_market(
        pool,
        source_market_id,
        question,
        &slug,
        None,
    ).await?;

    Ok(())
}

fn slug_from_title(title: &str) -> String {
    title.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(200)
        .collect()
}
