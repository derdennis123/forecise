use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use uuid::Uuid;

use crate::state::AppState;
use forecise_shared::models::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{market_id}", get(get_consensus))
        .route("/{market_id}/history", get(get_consensus_history))
}

async fn get_consensus(
    State(state): State<AppState>,
    Path(market_id): Path<Uuid>,
) -> impl IntoResponse {
    let consensus = sqlx::query_as::<_, ConsensusSnapshot>(
        r#"
        SELECT * FROM consensus_snapshots
        WHERE market_id = $1
        ORDER BY time DESC
        LIMIT 1
        "#,
    )
    .bind(market_id)
    .fetch_optional(&state.db)
    .await;

    match consensus {
        Ok(Some(data)) => Json(ApiResponse::new(data)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "No consensus data found for this market"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get consensus: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch consensus"
                })),
            )
                .into_response()
        }
    }
}

async fn get_consensus_history(
    State(state): State<AppState>,
    Path(market_id): Path<Uuid>,
) -> impl IntoResponse {
    let history = sqlx::query_as::<_, ConsensusSnapshot>(
        r#"
        SELECT * FROM consensus_snapshots
        WHERE market_id = $1
        ORDER BY time DESC
        LIMIT 100
        "#,
    )
    .bind(market_id)
    .fetch_all(&state.db)
    .await;

    match history {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get consensus history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch consensus history"
                })),
            )
                .into_response()
        }
    }
}
