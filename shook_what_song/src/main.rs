use std::net::SocketAddr;

use gumdrop::Options;

pub mod helpers;
mod history;
mod spotify;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from(&[".dev.env", ".env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let args = Args::parse_args_default_or_exit();
    let key = std::env::var("YOUTUBE_API_KEY")?;

    let addr = format!("{}:{}", args.address, args.port);
    let addr = tokio::net::lookup_host(&addr).await?.next().unwrap();

    start_server(addr, &key).await
}

// TODO make this part of the bot, proper
async fn start_server(addr: SocketAddr, api_key: &str) -> anyhow::Result<()> {
    let youtube = youtube::router(api_key, "list.csv").await?;
    let spotify = spotify::router("spotify.csv").await?;
    let router = axum::Router::new()
        .nest("/youtube", youtube)
        .nest("/spotify", spotify);

    log::info!("listening on: {}", addr);
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
