use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use std::str::FromStr;

/// Upsert a source market and record its odds
pub async fn upsert_source_market(
    pool: &PgPool,
    source_slug: &str,
    external_id: &str,
    title: &str,
    probability: f64,
    volume: Option<f64>,
    external_url: Option<&str>,
    metadata: serde_json::Value,
) -> Result<Uuid> {
    let source_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM sources WHERE slug = $1"
    )
    .bind(source_slug)
    .fetch_one(pool)
    .await?;

    let prob = BigDecimal::from_str(&format!("{:.6}", probability))?;
    let vol = volume.map(|v| BigDecimal::from_str(&format!("{:.2}", v)).unwrap_or_default());

    // Upsert source market
    let source_market_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO source_markets (source_id, external_id, title, current_probability, volume, external_url, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (source_id, external_id) DO UPDATE SET
            title = EXCLUDED.title,
            current_probability = EXCLUDED.current_probability,
            volume = EXCLUDED.volume,
            external_url = EXCLUDED.external_url,
            metadata = EXCLUDED.metadata,
            updated_at = NOW()
        RETURNING id
        "#
    )
    .bind(source_id)
    .bind(external_id)
    .bind(title)
    .bind(&prob)
    .bind(&vol)
    .bind(external_url)
    .bind(&metadata)
    .fetch_one(pool)
    .await?;

    // Record odds history
    sqlx::query(
        r#"
        INSERT INTO odds_history (time, source_market_id, probability, volume)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(Utc::now())
    .bind(source_market_id)
    .bind(&prob)
    .bind(&vol)
    .execute(pool)
    .await?;

    Ok(source_market_id)
}

/// Create or find a unified market for a source market
pub async fn ensure_unified_market(
    pool: &PgPool,
    source_market_id: Uuid,
    title: &str,
    slug: &str,
    category_slug: Option<&str>,
) -> Result<Uuid> {
    // Check if source market already has a unified market
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT market_id FROM source_markets WHERE id = $1"
    )
    .bind(source_market_id)
    .fetch_one(pool)
    .await?;

    if let Some(market_id) = existing {
        return Ok(market_id);
    }

    let category_id: Option<Uuid> = if let Some(cat) = category_slug {
        sqlx::query_scalar("SELECT id FROM categories WHERE slug = $1")
            .bind(cat)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // Create unified market
    let market_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO markets (slug, title, category_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (slug) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#
    )
    .bind(slug)
    .bind(title)
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    // Link source market to unified market
    sqlx::query("UPDATE source_markets SET market_id = $1 WHERE id = $2")
        .bind(market_id)
        .bind(source_market_id)
        .execute(pool)
        .await?;

    Ok(market_id)
}
