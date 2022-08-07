use crate::{callable::SharedCallable, state::GlobalState};

mod bot;
mod connection;
mod message;
mod parser;
mod tags;
mod types;

use connection::Connection;
pub use message::Message;
pub use tags::Tags;
pub(crate) use types::{Identity, Privmsg};

pub async fn create_bot<const N: usize>(
    state: GlobalState,
    callables: [SharedCallable; N],
) -> anyhow::Result<()> {
    pub const TWITCH_NO_TLS: &str = "irc.chat.twitch.tv:6667";

    let reg = types::Registration {
        name: "shaken_bot",
        pass: &std::env::var("SHAKEN_TWITCH_OAUTH_TOKEN").unwrap(),
    };

    log::info!("connecting to twitch");
    let (identity, conn) = Connection::connect(TWITCH_NO_TLS, reg).await?;
    state.insert(identity).await;

    log::info!("connected");

    let mut bot = bot::Bot::new(conn, state, callables);
    log::info!("joining channel");
    bot.join("#museun").await?;

    log::info!("starting the twitch bot");
    bot.start().await?;
    Ok(())
}
