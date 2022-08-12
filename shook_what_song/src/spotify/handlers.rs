use super::state::{Item, State};
use crate::helpers::*;

use axum::{response::IntoResponse, Extension, Json};

#[derive(serde::Deserialize)]
pub struct Insert {
    id: String,
    title: String,
}

pub async fn insert(
    Json(Insert { id, title }): Json<Insert>,
    Extension(state): Extension<State>,
) -> impl IntoResponse {
    log::debug!("trying to add '{id}' ({title})");
    let item = Item { id, title };
    state.add(item).await.map_err(|e| e.to_string())
}

pub async fn current(Extension(state): Extension<State>) -> impl IntoResponse {
    match state.current().await {
        Some(item) => {
            log::debug!("current: {:?}", item);
            response(item)
        }
        None => not_found(),
    }
}

pub async fn previous(Extension(state): Extension<State>) -> impl IntoResponse {
    match state.previous().await {
        Some(item) => {
            log::debug!("previous: {:?}", item);
            response(item)
        }
        None => not_found(),
    }
}

pub async fn all(Extension(state): Extension<State>) -> impl IntoResponse {
    let items = state.all().await;
    if items.is_empty() {
        return not_found();
    }
    response(items)
}
