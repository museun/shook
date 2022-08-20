use std::path::Path;

use axum::{
    routing::{get, post},
    Extension,
};

use client::{Client, Item};
use state::State;

mod client;
mod handlers;
mod state;

pub async fn router(api_key: &str, path: &Path) -> anyhow::Result<axum::Router> {
    let state = State::new(Client::new(api_key), path).await?;
    Ok(axum::Router::new()
        .route("/", post(handlers::insert))
        .route("/current", get(handlers::current))
        .route("/previous", get(handlers::previous))
        .route("/all", get(handlers::all))
        .layer(Extension(state)))
}
