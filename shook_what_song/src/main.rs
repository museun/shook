use std::net::SocketAddr;

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
    #[options(default = "localhost")]
    address: String,

    /// port to listen on
    #[options(default = "58810")]
    port: u16,
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

    let port = get_env_var("SHAKEN_WHAT_SONG_PORT")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(args.port);

    let address = get_env_var("SHAKEN_WHAT_SONG_ADDRESS")
        .ok()
        .unwrap_or(args.address);

    let bearer = get_env_var("SHAKEN_WHAT_SONG_BEARER_TOKEN")?;

    let addr = format!("{}:{}", address, port);
    let addr = tokio::net::lookup_host(&addr).await?.next().unwrap();

    start_server(addr, &key, &bearer).await
}

async fn start_server(addr: SocketAddr, api_key: &str, bearer: &str) -> anyhow::Result<()> {
    let auth = RequireAuthorizationLayer::bearer(bearer);

    let youtube = youtube::router(api_key, "list.csv").await?;
    let router = axum::Router::new()
        .nest("/youtube", youtube)
        .route_layer(auth);

    log::info!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
