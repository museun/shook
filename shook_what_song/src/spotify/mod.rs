use axum::{
    routing::{get, post},
    Extension,
};

mod handlers;
mod state;

pub async fn router(path: &str) -> anyhow::Result<axum::Router> {
    let state = state::State::load(path).await?;
    Ok(axum::Router::new()
        .route("/", post(handlers::insert))
        .route("/current", get(handlers::current))
        .route("/previous", get(handlers::previous))
        .route("/all", get(handlers::all))
        .layer(Extension(state)))
}
