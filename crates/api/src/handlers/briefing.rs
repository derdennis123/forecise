use axum::{
    Router, Json,
    extract::{Path, State},
    routing::get,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

use forecise_shared::models::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/latest", get(get_latest_briefing))
        .route("/date/{date}", get(get_briefing_by_date))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct MorningBriefing {
    id: Uuid,
    briefing_date: NaiveDate,
    total_markets_tracked: i32,
    new_markets_24h: i32,
    top_movers: serde_json::Value,
    high_volume_markets: serde_json::Value,
    source_agreement: serde_json::Value,
    key_stats: serde_json::Value,
    summary: Option<String>,
    created_at: DateTime<Utc>,
}

async fn get_latest_briefing(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let briefing = sqlx::query_as::<_, MorningBriefing>(
        "SELECT * FROM morning_briefings ORDER BY briefing_date DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await;

    match briefing {
        Ok(Some(data)) => Json(ApiResponse::new(data)).into_response(),
        Ok(None) => {
            // Return a generated-on-the-fly briefing from current data
            match generate_live_briefing(&state.db).await {
                Ok(data) => Json(ApiResponse::new(data)).into_response(),
                Err(e) => {
                    tracing::error!("Failed to generate live briefing: {}", e);
                    (StatusCode::NOT_FOUND, Json(serde_json::json!({
                        "error": "No briefing available yet"
                    }))).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get briefing: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch briefing"
            }))).into_response()
        }
    }
}

async fn get_briefing_by_date(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid date format. Use YYYY-MM-DD"
            }))).into_response()
        }
    };

    let briefing = sqlx::query_as::<_, MorningBriefing>(
        "SELECT * FROM morning_briefings WHERE briefing_date = $1"
    )
    .bind(parsed_date)
    .fetch_optional(&state.db)
    .await;

    match briefing {
        Ok(Some(data)) => Json(ApiResponse::new(data)).into_response(),
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "No briefing found for this date"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get briefing: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch briefing"
            }))).into_response()
        }
    }
}

#[derive(Debug, Serialize)]
struct LiveBriefing {
    briefing_date: String,
    total_markets_tracked: i64,
    new_markets_24h: i64,
    top_movers: Vec<LiveMover>,
    high_volume_markets: Vec<LiveVolume>,
    source_counts: Vec<SourceCount>,
    summary: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LiveMover {
    market_id: Uuid,
    title: String,
    source_name: String,
    probability_before: bigdecimal::BigDecimal,
    probability_after: bigdecimal::BigDecimal,
    change_pct: bigdecimal::BigDecimal,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LiveVolume {
    market_id: Uuid,
    title: String,
    total_volume: Option<bigdecimal::BigDecimal>,
    source_count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct SourceCount {
    source_name: String,
    market_count: i64,
}

async fn generate_live_briefing(pool: &sqlx::PgPool) -> Result<LiveBriefing, anyhow::Error> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM markets WHERE status = 'active'"
    )
    .fetch_one(pool)
    .await?;

    let since = Utc::now() - chrono::Duration::hours(24);

    let new_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM markets WHERE created_at >= $1"
    )
    .bind(since)
    .fetch_one(pool)
    .await?;

    let top_movers = sqlx::query_as::<_, LiveMover>(
        r#"
        SELECT * FROM (
            SELECT DISTINCT ON (me.market_id)
                me.market_id, m.title, s.name as source_name,
                me.probability_before, me.probability_after, me.change_pct
            FROM movement_events me
            JOIN markets m ON me.market_id = m.id
            JOIN source_markets sm ON me.source_market_id = sm.id
            JOIN sources s ON sm.source_id = s.id
            WHERE me.detected_at >= $1
            ORDER BY me.market_id, me.change_pct DESC
        ) sub
        ORDER BY change_pct DESC
        LIMIT 15
        "#
    )
    .bind(since)
    .fetch_all(pool)
    .await?;

    let high_volume = sqlx::query_as::<_, LiveVolume>(
        r#"
        SELECT m.id as market_id, m.title,
            SUM(sm.volume) as total_volume,
            COUNT(sm.id) as source_count
        FROM markets m
        JOIN source_markets sm ON sm.market_id = m.id
        WHERE m.status = 'active' AND sm.volume > 0
        GROUP BY m.id, m.title
        ORDER BY total_volume DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    let source_counts = sqlx::query_as::<_, SourceCount>(
        r#"
        SELECT s.name as source_name, COUNT(sm.id) as market_count
        FROM sources s
        JOIN source_markets sm ON sm.source_id = s.id
        WHERE s.is_active = true
        GROUP BY s.name
        ORDER BY market_count DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let movers_count = top_movers.len();
    let summary = format!(
        "Tracking {} active markets. {} new markets in the last 24 hours. {} significant movements detected.",
        total, new_24h, movers_count
    );

    Ok(LiveBriefing {
        briefing_date: Utc::now().format("%Y-%m-%d").to_string(),
        total_markets_tracked: total,
        new_markets_24h: new_24h,
        top_movers,
        high_volume_markets: high_volume,
        source_counts,
        summary,
    })
}
