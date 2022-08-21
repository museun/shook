use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use gumdrop::Options;
use tower_http::auth::RequireAuthorizationLayer;

pub mod helpers;
mod history;
mod youtube;

#[derive(gumdrop::Options, Debug)]
struct Args {
    /// print this message
    help: bool,

    /// address to listen on
    #[options(default = "127.0.0.1:58810")]
    address: String,

    /// history file to use
    #[options(short = "f", meta = "<path>")]
    history_file: PathBuf,
}

fn get_env_var(key: &str) -> anyhow::Result<String> {
    std::env::var(key).or_else(|_| anyhow::bail!("cannot find env var for '{key}'"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from(&[".dev.env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let args = Args::parse_args_default_or_exit();
    let key = get_env_var("SHAKEN_YOUTUBE_API_KEY")?;

    let history_file = get_env_var("SHAKEN_WHAT_SONG_HISTORY_FILE")
        .ok()
        .map(PathBuf::from)
        .unwrap_or(args.history_file);

    let addr = get_env_var("SHAKEN_WHAT_SONG_REMOTE")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(args.address);

    let bearer = get_env_var("SHAKEN_WHAT_SONG_BEARER_TOKEN")?;

    start_server(addr.parse()?, &history_file, &key, &bearer).await
}

async fn start_server(
    addr: SocketAddr,
    history_file: &Path,
    api_key: &str,
    bearer: &str,
) -> anyhow::Result<()> {
    let youtube = youtube::router(api_key, history_file).await?;
    let router = axum::Router::new()
        .nest("/youtube", youtube)
        .route_layer(RequireAuthorizationLayer::bearer(bearer));

    log::info!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
