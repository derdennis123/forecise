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
        .route("/trades/{market_id}", get(get_whale_trades))
        .route("/wallets", get(get_smart_wallets))
        .route("/wallet/{address}", get(get_wallet_detail))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct WhaleTradeView {
    id: Uuid,
    wallet_address: String,
    trade_type: String,
    position: String,
    amount: BigDecimal,
    price: Option<BigDecimal>,
    traded_at: DateTime<Utc>,
    wallet_accuracy: Option<BigDecimal>,
    is_smart_money: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TradeParams {
    pub min_amount: Option<f64>,
    pub limit: Option<i64>,
}

async fn get_whale_trades(
    State(state): State<AppState>,
    Path(market_id): Path<Uuid>,
    Query(params): Query<TradeParams>,
) -> impl IntoResponse {
    let min_amount = params.min_amount.unwrap_or(10000.0);
    let limit = params.limit.unwrap_or(50).min(200);

    let trades = sqlx::query_as::<_, WhaleTradeView>(
        r#"
        SELECT
            wt.id, wt.wallet_address, wt.trade_type,
            wt.position, wt.amount, wt.price, wt.traded_at,
            wa.accuracy_pct as wallet_accuracy,
            wa.is_smart_money
        FROM whale_trades wt
        JOIN source_markets sm ON wt.source_market_id = sm.id
        LEFT JOIN wallet_accuracy wa ON wt.wallet_address = wa.wallet_address
        WHERE sm.market_id = $1
        AND wt.amount >= $2
        ORDER BY wt.traded_at DESC
        LIMIT $3
        "#
    )
    .bind(market_id)
    .bind(min_amount)
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match trades {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get whale trades: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch whale trades"
            }))).into_response()
        }
    }
}

async fn get_smart_wallets(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let wallets = sqlx::query_as::<_, forecise_shared::models::WalletAccuracy>(
        r#"
        SELECT * FROM wallet_accuracy
        WHERE is_smart_money = true
        ORDER BY accuracy_pct DESC NULLS LAST
        LIMIT 50
        "#
    )
    .fetch_all(&state.db)
    .await;

    match wallets {
        Ok(data) => Json(ApiResponse::new(data)).into_response(),
        Err(e) => {
            tracing::error!("Failed to get smart wallets: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch wallets"
            }))).into_response()
        }
    }
}

async fn get_wallet_detail(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    let wallet = sqlx::query_as::<_, forecise_shared::models::WalletAccuracy>(
        "SELECT * FROM wallet_accuracy WHERE wallet_address = $1"
    )
    .bind(&address)
    .fetch_optional(&state.db)
    .await;

    match wallet {
        Ok(Some(data)) => Json(ApiResponse::new(data)).into_response(),
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Wallet not found"
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get wallet: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch wallet"
            }))).into_response()
        }
    }
}
