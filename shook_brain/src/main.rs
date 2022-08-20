use std::path::{Path, PathBuf};

use gumdrop::Options;

use shook_brain::{
    start_server, BrainExt, ManagedBrain, Messaging, GENERATE_TIMEOUT, SAVE_DURATION,
};
use shook_markov::Brain;

#[derive(Debug, Options)]
/// serves shaken's brains for the internet to consume
struct Config {
    /// print this help message
    help: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Options)]
enum Command {
    /// used for making a new brain
    Make(MakeOptions),
    /// used for running the server
    Serve(ServeOptions),
}

#[derive(Debug, Options)]
/// used for making a new brain
struct MakeOptions {
    /// print this help message
    help: bool,

    /// path of brain to use
    #[options(required, meta = "<path>")]
    file: PathBuf,

    /// the name of the new brain"
    #[options(meta = "<string>", required)]
    name: String,

    /// ngram depth of the new brain
    #[options(meta = "<int>", default = "5")]
    depth: usize,
}

#[derive(Debug, Options)]
/// used for running the server
struct ServeOptions {
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
    simple_env_load::load_env_from([".dev.env", ".prod.env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let config = Config::parse_args_default_or_exit();

    let opts = match config.command {
        Some(Command::Make(make)) => {
            Brain::new(&make.name, make.depth).save(&make.file)?;
            eprintln!(
                "created brain '{}' ({}) at {}",
                &make.name,
                make.depth,
                make.file.display()
            );
            std::process::exit(0)
        }
        Some(Command::Serve(serve)) => serve,
        None => {
            eprintln!("{}", Config::usage());
            eprintln!("\nAvailable commands:");
            eprintln!("{}", Config::command_list().unwrap());
            std::process::exit(1)
        }
    };

    let bearer = std::env::var("SHAKEN_BRAIN_SUPER_SECRET_BEARER_TOKEN")?;

    log::info!("loading brain from {}", opts.file.display());
    let brain = load(&opts.file).await?;
    log::debug!("loaded brain");

    start_server(opts.address, brain, &bearer).await
}
