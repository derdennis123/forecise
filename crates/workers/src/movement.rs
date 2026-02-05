//! Movement Detection
//! Detects significant probability changes and records them.

use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

/// Minimum probability change to trigger a movement event (15% = 0.15)
const MOVEMENT_THRESHOLD: f64 = 0.05;

/// Check for significant movements across all active source markets
pub async fn detect_movements(pool: &PgPool) -> Result<usize> {
    let mut count = 0;

    // Get all active source markets with their previous probability
    let markets = sqlx::query_as::<_, MovementCheck>(
        r#"
        SELECT
            sm.id as source_market_id,
            sm.market_id,
            sm.current_probability,
            (
                SELECT oh.probability
                FROM odds_history oh
                WHERE oh.source_market_id = sm.id
                ORDER BY oh.time DESC
                OFFSET 1
                LIMIT 1
            ) as previous_probability
        FROM source_markets sm
        WHERE sm.status = 'active'
        AND sm.current_probability IS NOT NULL
        AND sm.market_id IS NOT NULL
        "#
    )
    .fetch_all(pool)
    .await?;

    for market in &markets {
        let current = market.current_probability.as_ref()
            .and_then(|p| p.to_string().parse::<f64>().ok())
            .unwrap_or(0.0);
        let previous = market.previous_probability.as_ref()
            .and_then(|p| p.to_string().parse::<f64>().ok())
            .unwrap_or(current);

        let change = (current - previous).abs();

        if change >= MOVEMENT_THRESHOLD {
            if let Some(market_id) = &market.market_id {
                let change_pct = BigDecimal::from_str(&format!("{:.4}", change))?;
                let prob_before = BigDecimal::from_str(&format!("{:.6}", previous))?;
                let prob_after = BigDecimal::from_str(&format!("{:.6}", current))?;

                sqlx::query(
                    r#"
                    INSERT INTO movement_events
                        (source_market_id, market_id, probability_before, probability_after, change_pct, detected_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#
                )
                .bind(market.source_market_id)
                .bind(market_id)
                .bind(&prob_before)
                .bind(&prob_after)
                .bind(&change_pct)
                .bind(Utc::now())
                .execute(pool)
                .await?;

                let direction = if current > previous { "UP" } else { "DOWN" };
                info!(
                    "Movement detected: {} {:.1}% -> {:.1}% ({} {:.1}%)",
                    direction, previous * 100.0, current * 100.0, direction, change * 100.0
                );

                count += 1;
            }
        }
    }

    Ok(count)
}

#[derive(sqlx::FromRow)]
struct MovementCheck {
    source_market_id: Uuid,
    market_id: Option<Uuid>,
    current_probability: Option<BigDecimal>,
    previous_probability: Option<BigDecimal>,
}
