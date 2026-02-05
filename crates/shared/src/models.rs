use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Sources ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Source {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub source_type: String,
    pub api_base_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Categories ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Markets ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Market {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub resolution_value: Option<BigDecimal>,
    pub resolution_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Source Markets ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceMarket {
    pub id: Uuid,
    pub market_id: Option<Uuid>,
    pub source_id: Uuid,
    pub external_id: String,
    pub external_url: Option<String>,
    pub title: String,
    pub current_probability: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub liquidity: Option<BigDecimal>,
    pub status: String,
    pub resolution_value: Option<BigDecimal>,
    pub resolution_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Odds History ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OddsHistory {
    pub time: DateTime<Utc>,
    pub source_market_id: Uuid,
    pub probability: BigDecimal,
    pub volume: Option<BigDecimal>,
    pub trade_count: Option<i32>,
}

// ─── Accuracy ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccuracyRecord {
    pub id: Uuid,
    pub source_id: Uuid,
    pub category_id: Option<Uuid>,
    pub total_resolved: i32,
    pub correct_predictions: i32,
    pub brier_score: Option<BigDecimal>,
    pub accuracy_pct: Option<BigDecimal>,
    pub last_calculated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PredictionScore {
    pub id: Uuid,
    pub source_market_id: Uuid,
    pub source_id: Uuid,
    pub market_id: Uuid,
    pub category_id: Option<Uuid>,
    pub predicted_probability: BigDecimal,
    pub actual_outcome: BigDecimal,
    pub brier_score: BigDecimal,
    pub resolved_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ─── Consensus ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConsensusSnapshot {
    pub time: DateTime<Utc>,
    pub market_id: Uuid,
    pub consensus_probability: BigDecimal,
    pub confidence_score: Option<BigDecimal>,
    pub source_count: i32,
    pub agreement_score: Option<BigDecimal>,
    pub outlier_sources: serde_json::Value,
    pub weights: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ─── Movement Events ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MovementEvent {
    pub id: Uuid,
    pub source_market_id: Uuid,
    pub market_id: Uuid,
    pub probability_before: BigDecimal,
    pub probability_after: BigDecimal,
    pub change_pct: BigDecimal,
    pub detected_at: DateTime<Utc>,
    pub explanation: Option<String>,
    pub related_news: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ─── Whale Tracking ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WhaleTrade {
    pub id: Uuid,
    pub source_market_id: Option<Uuid>,
    pub wallet_address: String,
    pub trade_type: String,
    pub position: String,
    pub amount: BigDecimal,
    pub price: Option<BigDecimal>,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub traded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WalletAccuracy {
    pub wallet_address: String,
    pub total_trades: i32,
    pub resolved_trades: i32,
    pub correct_trades: i32,
    pub accuracy_pct: Option<BigDecimal>,
    pub total_volume: BigDecimal,
    pub pnl: BigDecimal,
    pub is_smart_money: bool,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── API Response Types ───

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketWithSources {
    #[serde(flatten)]
    pub market: Market,
    pub category: Option<Category>,
    pub sources: Vec<SourceMarketSummary>,
    pub consensus: Option<ConsensusInfo>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SourceMarketSummary {
    pub source_name: String,
    pub source_slug: String,
    pub probability: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub accuracy_pct: Option<BigDecimal>,
    pub external_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusInfo {
    pub probability: BigDecimal,
    pub confidence: Option<BigDecimal>,
    pub source_count: i32,
    pub agreement: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccuracyLeaderboardEntry {
    pub rank: i64,
    pub source_name: String,
    pub source_slug: String,
    pub accuracy_pct: Option<BigDecimal>,
    pub brier_score: Option<BigDecimal>,
    pub total_resolved: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MarketListItem {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub category_name: Option<String>,
    pub category_slug: Option<String>,
    pub status: String,
    pub consensus_probability: Option<BigDecimal>,
    pub source_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    pub fn with_pagination(data: T, page: i64, per_page: i64, total: i64) -> Self {
        Self {
            data,
            meta: Some(PaginationMeta {
                page,
                per_page,
                total,
                total_pages: (total + per_page - 1) / per_page,
            }),
        }
    }
}
