use axum::{
    Router, Json,
    extract::{Path, Query, State},
    routing::get,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

use forecise_shared::models::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{market_id}", get(get_movements))
        .route("/recent", get(get_recent_movements))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MovementWithContext {
    pub id: Uuid,
    pub market_id: Uuid,
    pub market_title: Option<String>,
    pub source_name: Option<String>,
    pub probability_before: BigDecimal,
    pub probability_after: BigDecimal,
    pub change_pct: BigDecimal,
    pub detected_at: DateTime<Utc>,
    pub explanation: Option<String>,
    pub related_news: serde_json::Value,
}

async fn get_movements(
    State(state): State<AppState>,
    Path(market_id): Path<Uuid>,
) -> impl IntoResponse {
    let movements = sqlx::query_as::<_, MovementWithContext>(
        r#"
        SELECT
            me.id, me.market_id,
            m.title as market_title,
            s.name as source_name,
            me.probability_before, me.probability_after,
            me.change_pct, me.detected_at,
            me.explanation, me.related_news
        FROM movement_events me
        JOIN markets m ON me.market_id = m.id
        JOIN source_markets sm ON me.source_market_id = sm.id
        JOIN sources s ON sm.source_id = s.id
        WHERE me.market_id = $1
        ORDER BY me.detected_at DESC
        LIMIT 50
        "#
    )
    .bind(market_id)
    .fetch_all(&state.db)
    .await;

    match movements {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get movements: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch movements"
            }))).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RecentParams {
    pub limit: Option<i64>,
}

async fn get_recent_movements(
    State(state): State<AppState>,
    Query(params): Query<RecentParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);

    let movements = sqlx::query_as::<_, MovementWithContext>(
        r#"
        SELECT
            me.id, me.market_id,
            m.title as market_title,
            s.name as source_name,
            me.probability_before, me.probability_after,
            me.change_pct, me.detected_at,
            me.explanation, me.related_news
        FROM movement_events me
        JOIN markets m ON me.market_id = m.id
        JOIN source_markets sm ON me.source_market_id = sm.id
        JOIN sources s ON sm.source_id = s.id
        ORDER BY me.detected_at DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match movements {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get recent movements: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch movements"
            }))).into_response()
        }
    }
}
