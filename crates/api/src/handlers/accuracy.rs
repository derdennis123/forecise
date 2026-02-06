use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;

use crate::state::AppState;
use forecise_shared::models::*;

pub fn routes() -> Router<AppState> {
    Router::new().route("/leaderboard", get(leaderboard))
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
    pub category: Option<String>,
    pub limit: Option<i64>,
}

async fn leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);

    // Try Redis cache
    let cache_key = format!(
        "accuracy:leaderboard:{}:{}",
        params.category.as_deref().unwrap_or(""),
        limit
    );
    if let Some(cached) = crate::cache::get::<serde_json::Value>(&state.redis, &cache_key).await {
        return Json(cached).into_response();
    }

    let entries = sqlx::query_as::<_, AccuracyLeaderboardEntry>(
        r#"
        SELECT
            ROW_NUMBER() OVER (ORDER BY ar.accuracy_pct DESC NULLS LAST) as rank,
            s.name as source_name,
            s.slug as source_slug,
            ar.accuracy_pct,
            ar.brier_score,
            ar.total_resolved
        FROM accuracy_records ar
        JOIN sources s ON ar.source_id = s.id
        WHERE ar.total_resolved >= 30
        AND ($1::text IS NULL OR ar.category_id = (SELECT id FROM categories WHERE slug = $1))
        ORDER BY ar.accuracy_pct DESC NULLS LAST
        LIMIT $2
        "#,
    )
    .bind(&params.category)
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match entries {
        Ok(data) => {
            let response = ApiResponse::new(data);
            crate::cache::set(&state.redis, &cache_key, &response, 60).await;
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get leaderboard: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch leaderboard"
                })),
            )
                .into_response()
        }
    }
}
