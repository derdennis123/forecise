use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn, error};

use crate::ingestion;

const METACULUS_API: &str = "https://www.metaculus.com/api2";
const POLL_INTERVAL_SECS: u64 = 600; // 10 minutes

#[derive(Debug, Deserialize)]
struct MetaculusResponse {
    results: Vec<MetaculusQuestion>,
    next: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetaculusQuestion {
    id: i64,
    title: Option<String>,
    url: Option<String>,
    status: Option<String>,
    community_prediction: Option<CommunityPrediction>,
    number_of_forecasters: Option<i64>,
    #[serde(rename = "type")]
    question_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommunityPrediction {
    full: Option<PredictionFull>,
}

#[derive(Debug, Deserialize)]
struct PredictionFull {
    q2: Option<f64>, // median
}

pub async fn run_worker(pool: PgPool, client: Client) -> Result<()> {
    info!("Starting Metaculus worker");

    // Initial delay to stagger workers
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    loop {
        match fetch_and_store(&pool, &client).await {
            Ok(count) => info!("Metaculus: ingested {} questions", count),
            Err(e) => error!("Metaculus worker error: {}", e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn fetch_and_store(pool: &PgPool, client: &Client) -> Result<usize> {
    let mut total = 0;
    let mut url = format!(
        "{}/questions/?limit=100&status=open&type=binary&order_by=-activity",
        METACULUS_API
    );

    for _ in 0..5 { // Max 5 pages
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("Metaculus API returned {}", response.status());
            break;
        }

        let data: MetaculusResponse = response.json().await?;

        for question in &data.results {
            if let Err(e) = process_question(pool, question).await {
                warn!("Failed to process Metaculus question {}: {}", question.id, e);
            } else {
                total += 1;
            }
        }

        match data.next {
            Some(next_url) => url = next_url,
            None => break,
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    Ok(total)
}

async fn process_question(pool: &PgPool, question: &MetaculusQuestion) -> Result<()> {
    let title = match &question.title {
        Some(t) => t,
        None => return Ok(()),
    };

    let probability = question.community_prediction
        .as_ref()
        .and_then(|cp| cp.full.as_ref())
        .and_then(|f| f.q2)
        .unwrap_or(0.5);

    let external_id = question.id.to_string();
    let external_url = question.url.as_deref()
        .or(Some(&format!("https://www.metaculus.com/questions/{}/", question.id)))
        .map(String::from);

    let metadata = serde_json::json!({
        "status": question.status,
        "question_type": question.question_type,
        "forecasters": question.number_of_forecasters,
    });

    let source_market_id = ingestion::upsert_source_market(
        pool,
        "metaculus",
        &external_id,
        title,
        probability,
        None,
        external_url.as_deref(),
        metadata,
    ).await?;

    let slug = format!("mc-{}", slug_from_title(title));

    ingestion::ensure_unified_market(
        pool,
        source_market_id,
        title,
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
