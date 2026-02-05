use axum::{
    Router, Json,
    extract::State,
    routing::post,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(ask_forecise))
}

#[derive(Debug, Deserialize)]
pub struct AskRequest {
    pub question: String,
}

#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub relevant_markets: Vec<RelevantMarket>,
    pub disclaimer: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RelevantMarket {
    pub id: Uuid,
    pub title: String,
    pub consensus_probability: Option<BigDecimal>,
    pub source_count: i64,
    pub relevance_score: f64,
}

async fn ask_forecise(
    State(state): State<AppState>,
    Json(req): Json<AskRequest>,
) -> impl IntoResponse {
    let query = req.question.trim();

    if query.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Question cannot be empty"
        }))).into_response();
    }

    // Search for relevant markets using full-text search
    let search_terms: Vec<&str> = query.split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();

    if search_terms.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Please provide a more specific question"
        }))).into_response();
    }

    // Build search pattern
    let search_pattern = search_terms.join(" | ");

    let markets = sqlx::query_as::<_, RelevantMarket>(
        r#"
        SELECT
            m.id,
            m.title,
            (
                SELECT cs.consensus_probability
                FROM consensus_snapshots cs
                WHERE cs.market_id = m.id
                ORDER BY cs.time DESC
                LIMIT 1
            ) as consensus_probability,
            COUNT(DISTINCT sm.id) as source_count,
            ts_rank(
                to_tsvector('english', m.title || ' ' || COALESCE(m.description, '')),
                plainto_tsquery('english', $1)
            )::float8 as relevance_score
        FROM markets m
        LEFT JOIN source_markets sm ON sm.market_id = m.id
        WHERE m.status = 'active'
        AND (
            m.title ILIKE '%' || $2 || '%'
            OR to_tsvector('english', m.title || ' ' || COALESCE(m.description, ''))
               @@ plainto_tsquery('english', $1)
        )
        GROUP BY m.id, m.title
        ORDER BY relevance_score DESC
        LIMIT 5
        "#
    )
    .bind(&search_pattern)
    .bind(search_terms.first().unwrap_or(&""))
    .fetch_all(&state.db)
    .await;

    match markets {
        Ok(relevant_markets) => {
            // Build a structured answer
            let answer = if relevant_markets.is_empty() {
                format!(
                    "I couldn't find any active prediction markets directly related to \"{}\". \
                     Try rephrasing your question or browse the markets page for available topics.",
                    query
                )
            } else {
                let mut parts = vec![format!(
                    "Based on {} relevant prediction market{}, here's what the data shows:\n",
                    relevant_markets.len(),
                    if relevant_markets.len() > 1 { "s" } else { "" }
                )];

                for market in &relevant_markets {
                    let prob_str = match &market.consensus_probability {
                        Some(p) => format!("{:.1}%", p.to_string().parse::<f64>().unwrap_or(0.0) * 100.0),
                        None => "No consensus yet".into(),
                    };
                    parts.push(format!(
                        "- **{}**: {} (from {} source{})",
                        market.title,
                        prob_str,
                        market.source_count,
                        if market.source_count > 1 { "s" } else { "" }
                    ));
                }

                parts.join("\n")
            };

            let response = AskResponse {
                answer,
                relevant_markets,
                disclaimer: "This is aggregated prediction market data, not financial advice. \
                            Prediction markets reflect crowd sentiment, not certainty.".into(),
            };

            Json(serde_json::json!({ "data": response })).into_response()
        }
        Err(e) => {
            tracing::error!("Ask Forecise error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to process question"
            }))).into_response()
        }
    }
}
