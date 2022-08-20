use std::path::{Path, PathBuf};

use gumdrop::Options;

use shook_brain::{start_server, ManagedBrain, Messaging, GENERATE_TIMEOUT, SAVE_DURATION};

#[derive(Debug, Options)]
/// serves shaken's brains for the internet to consume
struct Config {
    /// print this help message
    help: bool,

    /// path of brain to use
    #[options(required, meta = "<path>")]
    file: PathBuf,

    /// address to listen on
    #[options(default = "localhost:50000", meta = "<addr>")]
    address: String,
}

async fn load(path: &Path) -> anyhow::Result<Messaging> {
    let brain = shook_brain::load(path).await?;
    log::trace!("spawning brain handle thread");
    let msg = ManagedBrain::spawn(brain, path, GENERATE_TIMEOUT, SAVE_DURATION);
    Ok(Messaging::new(msg))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let config = Config::parse_args_default_or_exit();
    let bearer = std::env::var("SHAKEN_BRAIN_SUPER_SECRET_BEARER_TOKEN")?;

    log::info!("loading brain from {}", config.file.display());
    let brain = load(&config.file).await?;
    log::debug!("loaded brain");

    start_server(config.address, brain, &bearer).await
}
