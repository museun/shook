use anyhow::Context as _;
use axum::{
    routing::{get, post},
    Extension, Router, Server,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tower_http::auth::require_authorization::RequireAuthorizationLayer;

use shook_markov::Brain;

mod handlers;

mod messaging;
pub use messaging::Messaging;

mod request;
mod response;

mod managed;
pub use managed::ManagedBrain;

pub const SAVE_DURATION: Duration = Duration::from_secs(5 * 60);
pub const GENERATE_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn load(path: impl Into<PathBuf>) -> anyhow::Result<Brain> {
    let path = path.into();
    tokio::task::spawn_blocking(move || {
        let reader = std::io::BufReader::new(std::fs::File::open(path)?);
        let dec = zstd::Decoder::new(reader)?;
        let element = bincode::deserialize_from(dec)?;
        anyhow::Result::<_>::Ok(element)
    })
    .await
    .unwrap()
}

pub trait BrainExt {
    fn save(&self, path: &Path) -> anyhow::Result<()>;
}

impl BrainExt for Brain {
    fn save(&self, path: &Path) -> anyhow::Result<()> {
        log::debug!("saving brain to: {}", path.display());
        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        let enc = zstd::Encoder::new(writer, 0)?.auto_finish();
        bincode::serialize_into(enc, self)?;
        log::trace!("saved");
        Ok(())
    }
}

pub async fn start_server(
    addr: impl tokio::net::ToSocketAddrs + Send + 'static,
    messaging: Messaging,
    bearer: &str,
) -> anyhow::Result<()> {
    log::trace!("resolving hosts");
    let addr = tokio::net::lookup_host(addr)
        .await?
        .next()
        .with_context(|| "could not resolve an addr")?;

    let auth = RequireAuthorizationLayer::bearer(bearer);
    let app = Router::new()
        .route("/generate", get(handlers::generate))
        .merge(
            Router::new()
                .route("/train", post(handlers::train))
                .route("/save", post(handlers::save))
                .route_layer(auth),
        )
        .layer(Extension(messaging));

    log::info!("listening on host: {addr}");
    Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

trait InspectErr<E> {
    fn inspect_error<F>(self, f: F) -> Self
    where
        F: Fn(&E);
}

impl<T, E> InspectErr<E> for Result<T, E> {
    fn inspect_error<F>(self, f: F) -> Self
    where
        F: Fn(&E),
    {
        if let Err(err) = &self {
            f(err)
        }
        self
    }
}
