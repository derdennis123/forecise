use axum::{
    Router, Json,
    extract::{Path, State},
    routing::{get, post, delete},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;
use bigdecimal::BigDecimal;

use forecise_shared::models::*;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_alert))
        .route("/{alert_id}", delete(delete_alert))
        .route("/user/{user_id}", get(get_user_alerts))
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub alert_type: String,
    pub threshold_value: Option<f64>,
    pub channel: Option<String>,
}

async fn create_alert(
    State(state): State<AppState>,
    Json(req): Json<CreateAlertRequest>,
) -> impl IntoResponse {
    let threshold = req.threshold_value.map(|v| {
        use std::str::FromStr;
        BigDecimal::from_str(&format!("{:.4}", v)).unwrap_or_default()
    });

    let result = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO alerts (user_id, market_id, alert_type, threshold_value, channel)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
    .bind(req.user_id)
    .bind(req.market_id)
    .bind(&req.alert_type)
    .bind(&threshold)
    .bind(req.channel.as_deref().unwrap_or("email"))
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(id) => (StatusCode::CREATED, Json(serde_json::json!({
            "data": { "id": id, "status": "created" }
        }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create alert: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create alert"
            }))).into_response()
        }
    }
}

async fn delete_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM alerts WHERE id = $1")
        .bind(alert_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "data": { "deleted": true } })).into_response()
        }
        Ok(_) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Alert not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete alert: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to delete alert"
            }))).into_response()
        }
    }
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct AlertWithMarket {
    id: Uuid,
    market_id: Uuid,
    market_title: Option<String>,
    alert_type: String,
    threshold_value: Option<BigDecimal>,
    channel: String,
    is_active: bool,
    last_triggered_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn get_user_alerts(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let alerts = sqlx::query_as::<_, AlertWithMarket>(
        r#"
        SELECT
            a.id, a.market_id, m.title as market_title,
            a.alert_type, a.threshold_value, a.channel,
            a.is_active, a.last_triggered_at, a.created_at
        FROM alerts a
        JOIN markets m ON a.market_id = m.id
        WHERE a.user_id = $1
        ORDER BY a.created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await;

    match alerts {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get alerts: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch alerts"
            }))).into_response()
        }
    }
}
