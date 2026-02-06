use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;
use forecise_shared::models::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_markets))
        .route("/{id}", get(get_market))
        .route("/{id}/odds", get(get_market_odds))
        .route("/{id}/sources", get(get_market_sources))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

async fn list_markets(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Try Redis cache
    let cache_key = format!(
        "markets:list:{}:{}:{}:{}:{}",
        page,
        per_page,
        params.category.as_deref().unwrap_or(""),
        params.status.as_deref().unwrap_or(""),
        params.search.as_deref().unwrap_or("")
    );
    if let Some(cached) = crate::cache::get::<serde_json::Value>(&state.redis, &cache_key).await {
        return Json(cached).into_response();
    }

    let markets = sqlx::query_as::<_, MarketListItem>(
        r#"
        SELECT
            m.id, m.slug, m.title,
            c.name as category_name,
            c.slug as category_slug,
            m.status,
            cs.consensus_probability,
            COUNT(DISTINCT sm.id) as source_count,
            m.updated_at
        FROM markets m
        LEFT JOIN categories c ON m.category_id = c.id
        LEFT JOIN source_markets sm ON sm.market_id = m.id
        LEFT JOIN LATERAL (
            SELECT consensus_probability
            FROM consensus_snapshots
            WHERE market_id = m.id
            ORDER BY time DESC
            LIMIT 1
        ) cs ON true
        WHERE ($1::text IS NULL OR m.status = $1)
        AND ($2::text IS NULL OR c.slug = $2)
        AND ($3::text IS NULL OR m.title ILIKE '%' || $3 || '%')
        GROUP BY m.id, m.slug, m.title, c.name, c.slug, m.status,
                 cs.consensus_probability, m.updated_at
        ORDER BY m.updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(&params.status)
    .bind(&params.category)
    .bind(&params.search)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await;

    match markets {
        Ok(data) => {
            let total = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM markets WHERE ($1::text IS NULL OR status = $1)",
            )
            .bind(&params.status)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

            let response = ApiResponse::with_pagination(data, page, per_page, total);
            crate::cache::set(&state.redis, &cache_key, &response, 30).await;
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list markets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch markets"
                })),
            )
                .into_response()
        }
    }
}

async fn get_market(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let market = sqlx::query_as::<_, Market>("SELECT * FROM markets WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await;

    match market {
        Ok(Some(market)) => {
            // Fetch sources with accuracy info
            let sources = sqlx::query_as::<_, SourceMarketSummary>(
                r#"
                SELECT
                    s.name as source_name,
                    s.slug as source_slug,
                    sm.current_probability as probability,
                    sm.volume,
                    ar.accuracy_pct,
                    sm.external_url
                FROM source_markets sm
                JOIN sources s ON sm.source_id = s.id
                LEFT JOIN accuracy_records ar ON ar.source_id = s.id AND ar.category_id = $2
                WHERE sm.market_id = $1
                ORDER BY ar.accuracy_pct DESC NULLS LAST
                "#,
            )
            .bind(id)
            .bind(market.category_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let category = if let Some(cat_id) = market.category_id {
                sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
                    .bind(cat_id)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None)
            } else {
                None
            };

            // Latest consensus
            let consensus = sqlx::query_as::<_, ConsensusSnapshot>(
                r#"
                SELECT * FROM consensus_snapshots
                WHERE market_id = $1
                ORDER BY time DESC
                LIMIT 1
                "#,
            )
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None)
            .map(|cs| ConsensusInfo {
                probability: cs.consensus_probability,
                confidence: cs.confidence_score,
                source_count: cs.source_count,
                agreement: cs.agreement_score,
            });

            let response = MarketWithSources {
                market,
                category,
                sources,
                consensus,
            };

            Json(ApiResponse::new(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Market not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get market: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch market"
                })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OddsParams {
    pub timeframe: Option<String>, // "1d", "1w", "1m", "1y", "all"
    pub source: Option<String>,
}

async fn get_market_odds(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<OddsParams>,
) -> impl IntoResponse {
    let interval = match params.timeframe.as_deref() {
        Some("1d") => "1 day",
        Some("1w") => "7 days",
        Some("1m") => "30 days",
        Some("1y") => "365 days",
        _ => "30 days",
    };

    let odds = sqlx::query_as::<_, OddsHistory>(
        r#"
        SELECT oh.time, oh.source_market_id, oh.probability, oh.volume, oh.trade_count
        FROM odds_history oh
        JOIN source_markets sm ON oh.source_market_id = sm.id
        WHERE sm.market_id = $1
        AND oh.time > NOW() - $2::interval
        AND ($3::text IS NULL OR sm.source_id = (SELECT id FROM sources WHERE slug = $3))
        ORDER BY oh.time ASC
        "#,
    )
    .bind(id)
    .bind(interval)
    .bind(&params.source)
    .fetch_all(&state.db)
    .await;

    match odds {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get odds: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch odds history"
                })),
            )
                .into_response()
        }
    }
}

async fn get_market_sources(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let sources = sqlx::query_as::<_, SourceMarket>(
        "SELECT * FROM source_markets WHERE market_id = $1",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await;

    match sources {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get sources: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch sources"
                })),
            )
                .into_response()
        }
    }
}
