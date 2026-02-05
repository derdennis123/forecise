use axum::Router;

use crate::state::AppState;

pub mod accuracy;
pub mod consensus;
pub mod health;
pub mod markets;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .nest("/markets", markets::routes())
        .nest("/accuracy", accuracy::routes())
        .nest("/consensus", consensus::routes())
}
