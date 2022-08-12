use axum::{
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;

pub fn response<T>(item: T) -> Response
where
    T: serde::Serialize,
{
    Json(item).into_response()
}

pub fn not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
}
