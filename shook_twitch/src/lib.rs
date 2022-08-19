use shook_core::prelude::{GlobalState, SharedCallable};

pub mod config;

mod bot;
mod connection;
mod message;
mod parser;
mod tags;
mod types;

use connection::Connection;
pub use message::Message;
pub use tags::Tags;
pub use types::{Identity, Privmsg};

pub async fn create_bot<const N: usize>(
    state: GlobalState,
    callables: [SharedCallable; N],
) -> anyhow::Result<()> {
    let config: crate::config::Config = state.get_owned().await;

    let reg = types::Registration {
        name: &config.name,
        pass: &config.password,
    };

    log::info!(
        "connecting to {} (with name {})",
        &config.address,
        &config.name
    );
    let (identity, conn) = Connection::connect(&config.address, reg).await?;
    state.insert(identity).await;

    log::info!("connected");

    let mut bot = bot::Bot::new(conn, state, callables);
    log::info!("joining {}", &config.channel);
    bot.join(&config.channel).await?;

    log::info!("starting the twitch bot");
    bot.start().await?;
    Ok(())
}
