use std::sync::Arc;

use crate::{binding::Callable, state::SharedState};

mod connection;
pub use connection::Connection;

mod parser;

mod tags;
pub use tags::Tags;

mod types;
pub use types::{Identity, Privmsg};

pub async fn create_bot<const N: usize>(
    state: SharedState,
    callables: [Arc<Callable>; N],
) -> anyhow::Result<()> {
    pub const TWITCH_NO_TLS: &str = "irc.chat.twitch.tv:6667";

    let reg = types::Registration {
        name: "shaken_bot",
        pass: &std::env::var("SHAKEN_TWITCH_OAUTH_TOKEN").unwrap(),
    };

    log::info!("connecting to twitch");
    let (identity, conn) = Connection::connect(TWITCH_NO_TLS, reg).await?;
    log::info!("connected");

    let mut bot = bot::Bot::new(conn, state, callables);

    log::info!("joining channel");
    bot.join("#museun").await?;

    log::info!("starting the bot");
    bot.start().await?;
    Ok(())
}

mod bot;
