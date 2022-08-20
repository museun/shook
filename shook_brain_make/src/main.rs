use std::path::{Path, PathBuf};

use gumdrop::Options;
use shook_markov::Brain;

// TODO train
#[derive(Debug, Options)]
/// serves shaken's brains for the internet to consume
struct Config {
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

fn main() -> anyhow::Result<()> {
    let config: Config = Config::parse_args_default_or_exit();
    Brain::new(&config.name, config.depth).save(&config.file)?;
    eprintln!(
        "created brain '{}' ({}) at {}",
        &config.name,
        config.depth,
        config.file.display()
    );
    std::process::exit(0)
}
