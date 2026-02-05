//! Morning Briefing Generator
//! Generates a daily summary of prediction market activity.

use anyhow::Result;
use chrono::{Utc, Duration};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct TopMover {
    market_id: String,
    title: String,
    source_name: String,
    probability_before: f64,
    probability_after: f64,
    change_pct: f64,
    direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HighVolumeMarket {
    market_id: String,
    title: String,
    probability: f64,
    total_volume: f64,
    source_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceAgreement {
    market_id: String,
    title: String,
    min_probability: f64,
    max_probability: f64,
    spread: f64,
    source_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyStats {
    total_active_markets: i64,
    total_sources_active: i64,
    avg_source_count_per_market: f64,
    total_movements_24h: i64,
    markets_with_consensus: i64,
}

pub async fn run_briefing_generator(pool: PgPool) -> Result<()> {
    // Wait for initial data + consensus computation
    tokio::time::sleep(std::time::Duration::from_secs(120)).await;

    loop {
        match generate_briefing(&pool).await {
            Ok(true) => info!("Morning briefing generated successfully"),
            Ok(false) => {} // already generated today
            Err(e) => warn!("Briefing generation error: {}", e),
        }
        // Check every hour
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

async fn generate_briefing(pool: &PgPool) -> Result<bool> {
    let today = Utc::now().date_naive();

    // Check if already generated today
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM morning_briefings WHERE briefing_date = $1)"
    )
    .bind(today)
    .fetch_one(pool)
    .await?;

    if exists {
        return Ok(false);
    }

    let since = Utc::now() - Duration::hours(24);

    // 1. Top movers (biggest movements in last 24h)
    let top_movers = get_top_movers(pool, since).await?;

    // 2. High volume markets
    let high_volume = get_high_volume_markets(pool).await?;

    // 3. Source agreement / disagreement
    let source_agreement = get_source_agreement(pool).await?;

    // 4. Key stats
    let key_stats = get_key_stats(pool, since).await?;

    // 5. New markets in last 24h
    let new_markets_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM markets WHERE created_at >= $1"
    )
    .bind(since)
    .fetch_one(pool)
    .await?;

    // Generate summary text
    let summary = generate_summary_text(&top_movers, &high_volume, &key_stats, new_markets_24h);

    // Insert briefing
    sqlx::query(
        r#"
        INSERT INTO morning_briefings
            (briefing_date, total_markets_tracked, new_markets_24h, top_movers, high_volume_markets, source_agreement, key_stats, summary)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (briefing_date) DO UPDATE SET
            total_markets_tracked = EXCLUDED.total_markets_tracked,
            new_markets_24h = EXCLUDED.new_markets_24h,
            top_movers = EXCLUDED.top_movers,
            high_volume_markets = EXCLUDED.high_volume_markets,
            source_agreement = EXCLUDED.source_agreement,
            key_stats = EXCLUDED.key_stats,
            summary = EXCLUDED.summary
        "#
    )
    .bind(today)
    .bind(key_stats.total_active_markets as i32)
    .bind(new_markets_24h as i32)
    .bind(serde_json::to_value(&top_movers)?)
    .bind(serde_json::to_value(&high_volume)?)
    .bind(serde_json::to_value(&source_agreement)?)
    .bind(serde_json::to_value(&key_stats)?)
    .bind(&summary)
    .execute(pool)
    .await?;

    Ok(true)
}

async fn get_top_movers(pool: &PgPool, since: chrono::DateTime<Utc>) -> Result<Vec<TopMover>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        market_id: uuid::Uuid,
        title: String,
        source_name: String,
        probability_before: bigdecimal::BigDecimal,
        probability_after: bigdecimal::BigDecimal,
        change_pct: bigdecimal::BigDecimal,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT DISTINCT ON (me.market_id)
            me.market_id,
            m.title,
            s.name as source_name,
            me.probability_before,
            me.probability_after,
            me.change_pct
        FROM movement_events me
        JOIN markets m ON me.market_id = m.id
        JOIN source_markets sm ON me.source_market_id = sm.id
        JOIN sources s ON sm.source_id = s.id
        WHERE me.detected_at >= $1
        ORDER BY me.market_id, me.change_pct DESC
        "#
    )
    .bind(since)
    .fetch_all(pool)
    .await?;

    let mut movers: Vec<TopMover> = rows.into_iter().map(|r| {
        let before: f64 = r.probability_before.to_string().parse().unwrap_or(0.0);
        let after: f64 = r.probability_after.to_string().parse().unwrap_or(0.0);
        let change: f64 = r.change_pct.to_string().parse().unwrap_or(0.0);
        TopMover {
            market_id: r.market_id.to_string(),
            title: r.title,
            source_name: r.source_name,
            probability_before: before,
            probability_after: after,
            change_pct: change,
            direction: if after > before { "UP".to_string() } else { "DOWN".to_string() },
        }
    }).collect();

    movers.sort_by(|a, b| b.change_pct.partial_cmp(&a.change_pct).unwrap_or(std::cmp::Ordering::Equal));
    movers.truncate(10);
    Ok(movers)
}

async fn get_high_volume_markets(pool: &PgPool) -> Result<Vec<HighVolumeMarket>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        market_id: uuid::Uuid,
        title: String,
        avg_prob: Option<bigdecimal::BigDecimal>,
        total_volume: Option<bigdecimal::BigDecimal>,
        source_count: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            m.id as market_id,
            m.title,
            AVG(sm.current_probability) as avg_prob,
            SUM(sm.volume) as total_volume,
            COUNT(sm.id) as source_count
        FROM markets m
        JOIN source_markets sm ON sm.market_id = m.id
        WHERE m.status = 'active'
        AND sm.volume IS NOT NULL
        AND sm.volume > 0
        GROUP BY m.id, m.title
        ORDER BY total_volume DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        HighVolumeMarket {
            market_id: r.market_id.to_string(),
            title: r.title,
            probability: r.avg_prob.and_then(|p| p.to_string().parse().ok()).unwrap_or(0.5),
            total_volume: r.total_volume.and_then(|v| v.to_string().parse().ok()).unwrap_or(0.0),
            source_count: r.source_count,
        }
    }).collect())
}

async fn get_source_agreement(pool: &PgPool) -> Result<Vec<SourceAgreement>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        market_id: uuid::Uuid,
        title: String,
        min_prob: Option<bigdecimal::BigDecimal>,
        max_prob: Option<bigdecimal::BigDecimal>,
        source_count: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            m.id as market_id,
            m.title,
            MIN(sm.current_probability) as min_prob,
            MAX(sm.current_probability) as max_prob,
            COUNT(sm.id) as source_count
        FROM markets m
        JOIN source_markets sm ON sm.market_id = m.id
        WHERE m.status = 'active'
        AND sm.current_probability IS NOT NULL
        GROUP BY m.id, m.title
        HAVING COUNT(sm.id) >= 2
        ORDER BY (MAX(sm.current_probability) - MIN(sm.current_probability)) DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| {
        let min_p: f64 = r.min_prob.and_then(|p| p.to_string().parse().ok()).unwrap_or(0.0);
        let max_p: f64 = r.max_prob.and_then(|p| p.to_string().parse().ok()).unwrap_or(0.0);
        SourceAgreement {
            market_id: r.market_id.to_string(),
            title: r.title,
            min_probability: min_p,
            max_probability: max_p,
            spread: max_p - min_p,
            source_count: r.source_count,
        }
    }).collect())
}

async fn get_key_stats(pool: &PgPool, since: chrono::DateTime<Utc>) -> Result<KeyStats> {
    let total_active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM markets WHERE status = 'active'"
    )
    .fetch_one(pool)
    .await?;

    let total_sources: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sources WHERE is_active = true"
    )
    .fetch_one(pool)
    .await?;

    let avg_sources: f64 = sqlx::query_scalar::<_, Option<bigdecimal::BigDecimal>>(
        r#"
        SELECT AVG(cnt)::DECIMAL FROM (
            SELECT COUNT(sm.id) as cnt
            FROM markets m
            JOIN source_markets sm ON sm.market_id = m.id
            WHERE m.status = 'active'
            GROUP BY m.id
        ) sub
        "#
    )
    .fetch_one(pool)
    .await?
    .and_then(|v| v.to_string().parse().ok())
    .unwrap_or(1.0);

    let movements_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM movement_events WHERE detected_at >= $1"
    )
    .bind(since)
    .fetch_one(pool)
    .await?;

    let with_consensus: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT market_id) FROM consensus_snapshots
        WHERE time >= NOW() - INTERVAL '1 day'
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(KeyStats {
        total_active_markets: total_active,
        total_sources_active: total_sources,
        avg_source_count_per_market: avg_sources,
        total_movements_24h: movements_24h,
        markets_with_consensus: with_consensus,
    })
}

fn generate_summary_text(
    top_movers: &[TopMover],
    high_volume: &[HighVolumeMarket],
    stats: &KeyStats,
    new_markets: i64,
) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Tracking {} active markets across {} sources.",
        stats.total_active_markets, stats.total_sources_active
    ));

    if new_markets > 0 {
        lines.push(format!("{} new markets added in the last 24 hours.", new_markets));
    }

    if stats.total_movements_24h > 0 {
        lines.push(format!(
            "{} significant price movements detected.",
            stats.total_movements_24h
        ));
    }

    if let Some(mover) = top_movers.first() {
        lines.push(format!(
            "Biggest mover: \"{}\" moved {} {:.1}% (from {:.0}% to {:.0}%).",
            truncate_title(&mover.title, 60),
            mover.direction,
            mover.change_pct * 100.0,
            mover.probability_before * 100.0,
            mover.probability_after * 100.0,
        ));
    }

    if let Some(vol) = high_volume.first() {
        if vol.total_volume > 0.0 {
            lines.push(format!(
                "Highest volume: \"{}\" with ${:.0} traded.",
                truncate_title(&vol.title, 60),
                vol.total_volume,
            ));
        }
    }

    if stats.markets_with_consensus > 0 {
        lines.push(format!(
            "Consensus computed for {} markets.",
            stats.markets_with_consensus
        ));
    }

    lines.join(" ")
}

fn truncate_title(title: &str, max_len: usize) -> String {
    if title.len() <= max_len {
        title.to_string()
    } else {
        format!("{}...", &title[..max_len - 3])
    }
}
