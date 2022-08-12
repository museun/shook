use super::State;
use crate::helpers::*;
use axum::{response::IntoResponse, Extension, Json};

#[derive(serde::Deserialize)]
pub(super) struct Insert {
    id: String,
    ts: i64,
}

pub(super) async fn insert(
    Json(Insert { id, ts }): Json<Insert>,
    Extension(state): Extension<State>,
) -> impl IntoResponse {
    log::debug!("trying to add '{id}' ({ts})");
    state.add(&id, ts).await.map_err(|s| s.to_string())
}

pub(super) async fn current(Extension(state): Extension<State>) -> impl IntoResponse {
    match state.history.current().await {
        Some(item) => {
            log::debug!("current: {:?}", item);
            response(item)
        }
        None => not_found(),
    }
}

pub(super) async fn previous(Extension(state): Extension<State>) -> impl IntoResponse {
    match state.history.previous().await {
        Some(item) => {
            log::debug!("previous: {:?}", item);
            response(item)
        }
        None => not_found(),
    }
}

pub(super) async fn all(Extension(state): Extension<State>) -> impl IntoResponse {
    let items = state.history.all().await;
    if items.is_empty() {
        return not_found();
    }
    response(items)
}
