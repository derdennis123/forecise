use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;
use tracing::{info, warn};

use forecise_consensus::engine::{self, SourceInput};

pub async fn run_consensus_worker(pool: PgPool) -> Result<()> {
    // Wait for initial data
    tokio::time::sleep(std::time::Duration::from_secs(90)).await;

    loop {
        match compute_all_consensus(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Computed consensus for {} markets", count);
                }
            }
            Err(e) => warn!("Consensus computation error: {}", e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}

async fn compute_all_consensus(pool: &PgPool) -> Result<usize> {
    let market_ids: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT m.id
        FROM markets m
        JOIN source_markets sm ON sm.market_id = m.id
        WHERE m.status = 'active'
        AND sm.current_probability IS NOT NULL
        GROUP BY m.id
        HAVING COUNT(sm.id) >= 1
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut count = 0;
    for market_id in market_ids {
        if let Err(e) = compute_market_consensus(pool, market_id).await {
            warn!("Failed consensus for market {}: {}", market_id, e);
        } else {
            count += 1;
        }
    }

    Ok(count)
}

async fn compute_market_consensus(pool: &PgPool, market_id: Uuid) -> Result<()> {
    #[derive(sqlx::FromRow)]
    struct SourceData {
        source_slug: String,
        source_name: String,
        probability: BigDecimal,
        volume: Option<BigDecimal>,
        accuracy_pct: Option<BigDecimal>,
        total_resolved: Option<i32>,
    }

    let sources = sqlx::query_as::<_, SourceData>(
        r#"
        SELECT
            s.slug as source_slug,
            s.name as source_name,
            sm.current_probability as probability,
            sm.volume,
            ar.accuracy_pct,
            ar.total_resolved
        FROM source_markets sm
        JOIN sources s ON sm.source_id = s.id
        LEFT JOIN accuracy_records ar ON ar.source_id = s.id
        WHERE sm.market_id = $1
        AND sm.current_probability IS NOT NULL
        "#
    )
    .bind(market_id)
    .fetch_all(pool)
    .await?;

    if sources.is_empty() {
        return Ok(());
    }

    let inputs: Vec<SourceInput> = sources.iter().map(|s| {
        SourceInput {
            source_id: s.source_slug.clone(),
            source_name: s.source_name.clone(),
            probability: s.probability.to_string().parse().unwrap_or(0.5),
            accuracy_pct: s.accuracy_pct.as_ref().and_then(|a| a.to_string().parse().ok()),
            resolved_count: s.total_resolved.unwrap_or(0),
            volume: s.volume.as_ref().and_then(|v| v.to_string().parse().ok()),
        }
    }).collect();

    let result = engine::calculate_consensus(&inputs)?;

    let prob = BigDecimal::from_str(&format!("{:.6}", result.probability))?;
    let confidence = BigDecimal::from_str(&format!("{:.4}", result.confidence))?;
    let agreement = BigDecimal::from_str(&format!("{:.4}", result.agreement))?;
    let weights_json = serde_json::to_value(&result.weights)?;
    let outliers_json = serde_json::to_value(&result.outliers)?;

    sqlx::query(
        r#"
        INSERT INTO consensus_snapshots
            (time, market_id, consensus_probability, confidence_score, source_count, agreement_score, weights, outlier_sources)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(Utc::now())
    .bind(market_id)
    .bind(&prob)
    .bind(&confidence)
    .bind(result.source_count as i32)
    .bind(&agreement)
    .bind(&weights_json)
    .bind(&outliers_json)
    .execute(pool)
    .await?;

    Ok(())
}
