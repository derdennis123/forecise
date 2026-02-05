use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn, error};

use crate::ingestion;

const POLYMARKET_API: &str = "https://clob.polymarket.com";
const GAMMA_API: &str = "https://gamma-api.polymarket.com";
const POLL_INTERVAL_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Deserialize)]
struct GammaMarket {
    #[serde(rename = "conditionId")]
    condition_id: Option<String>,
    question: Option<String>,
    #[serde(rename = "outcomePrices")]
    outcome_prices: Option<String>,
    #[serde(rename = "volumeNum")]
    volume_num: Option<f64>,
    #[serde(rename = "liquidityNum")]
    liquidity_num: Option<f64>,
    slug: Option<String>,
    active: Option<bool>,
    closed: Option<bool>,
    #[serde(rename = "questionID")]
    question_id: Option<String>,
}

pub async fn run_worker(pool: PgPool, client: Client) -> Result<()> {
    info!("Starting Polymarket worker");

    loop {
        match fetch_and_store(&pool, &client).await {
            Ok(count) => info!("Polymarket: ingested {} markets", count),
            Err(e) => error!("Polymarket worker error: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn fetch_and_store(pool: &PgPool, client: &Client) -> Result<usize> {
    let mut offset = 0;
    let limit = 100;
    let mut total = 0;

    loop {
        let url = format!(
            "{}/markets?limit={}&offset={}&active=true&closed=false",
            GAMMA_API, limit, offset
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("Polymarket API returned {}", response.status());
            break;
        }

        let markets: Vec<GammaMarket> = response.json().await?;

        if markets.is_empty() {
            break;
        }

        for market in &markets {
            if let Err(e) = process_market(pool, market).await {
                warn!("Failed to process Polymarket market: {}", e);
            } else {
                total += 1;
            }
        }

        if markets.len() < limit {
            break;
        }

        offset += limit;

        // Rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(total)
}

async fn process_market(pool: &PgPool, market: &GammaMarket) -> Result<()> {
    let question = match &market.question {
        Some(q) => q,
        None => return Ok(()),
    };

    let external_id = market.condition_id.as_deref()
        .or(market.question_id.as_deref())
        .unwrap_or_default();

    if external_id.is_empty() {
        return Ok(());
    }

    // Parse outcome prices (JSON string like "[\"0.65\", \"0.35\"]")
    let probability = if let Some(prices_str) = &market.outcome_prices {
        let prices: Vec<String> = serde_json::from_str(prices_str).unwrap_or_default();
        prices.first()
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.5)
    } else {
        0.5
    };

    let external_url = market.slug.as_ref()
        .map(|s| format!("https://polymarket.com/event/{}", s));

    let metadata = serde_json::json!({
        "active": market.active,
        "closed": market.closed,
        "liquidity": market.liquidity_num,
    });

    let source_market_id = ingestion::upsert_source_market(
        pool,
        "polymarket",
        external_id,
        question,
        probability,
        market.volume_num,
        external_url.as_deref(),
        metadata,
    ).await?;

    // Create slug from question
    let slug = format!("pm-{}", slug_from_title(question));

    ingestion::ensure_unified_market(
        pool,
        source_market_id,
        question,
        &slug,
        None, // TODO: category detection
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
