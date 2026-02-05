use axum::Router;

use crate::state::AppState;

pub mod accuracy;
pub mod alerts;
pub mod ask;
pub mod briefing;
pub mod consensus;
pub mod health;
pub mod markets;
pub mod movements;
pub mod whales;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .nest("/markets", markets::routes())
        .nest("/accuracy", accuracy::routes())
        .nest("/ask", ask::routes())
        .nest("/consensus", consensus::routes())
        .nest("/movements", movements::routes())
        .nest("/alerts", alerts::routes())
        .nest("/whales", whales::routes())
        .nest("/briefing", briefing::routes())
}
