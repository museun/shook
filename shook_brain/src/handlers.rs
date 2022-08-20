use axum::{
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
    Extension, Json,
};

use crate::{
    messaging::{Request, Response},
    request, response, Messaging,
};

pub async fn generate(
    req: Option<Json<request::Generate>>,
    state: Extension<Messaging>,
) -> AxumResponse {
    let opts = req.map(|Json(data)| data).unwrap_or_default();
    log::debug!("request generate: {:?}", opts);
    match state.send(Request::Generate { opts }).await {
        Response::Generated { data } => {
            log::trace!("generated: {}", data.escape_debug());
            json(response::Generate { data })
        }
        Response::Error { error } => {
            log::warn!("could not generate: {error}");
            make_error(503, error)
        }
        _ => ok(),
    }
}

pub async fn train(
    Json(request::Train { data }): Json<request::Train>,
    state: Extension<Messaging>,
) -> impl IntoResponse {
    log::debug!("req train: {}", data.escape_debug());

    if let Response::Error { error } = state.send(Request::Train { data }).await {
        log::warn!("could not train: {error}");
        return make_error(401, error);
    }
    if let Response::Error { error } = state.send(Request::Save).await {
        log::warn!("could not save: {error}");
        return make_error(503, error);
    }
    ok()
}

pub async fn save(state: Extension<Messaging>) -> AxumResponse {
    log::debug!("req save");

    if let Response::Error { error } = state.send(Request::ForceSave).await {
        log::warn!("could not save: {error}");
        return make_error(503, error);
    }
    ok()
}

fn ok() -> AxumResponse {
    StatusCode::OK.into_response()
}

fn json<T: serde::Serialize + 'static + Send>(data: T) -> AxumResponse {
    Json(data).into_response()
}

fn make_error(code: u16, err: impl ToString + Send) -> AxumResponse {
    let status_code = StatusCode::from_u16(code).expect("valid status code");
    let json = json(response::Error {
        msg: err.to_string(),
    });
    (status_code, json).into_response()
}
