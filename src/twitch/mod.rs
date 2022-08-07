use std::sync::Arc;

use crate::{callable::SharedCallable, state::GlobalState};

mod bot;

mod connection;
use connection::Connection;

mod parser;

mod tags;
pub use tags::Tags;

mod types;
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

pub struct Message {
    pub(crate) sender: Arc<str>,
    target: Arc<str>,
    pub(crate) data: Arc<str>,
    tags: Arc<Tags>,
}

impl Message {
    pub fn from_pm(pm: Privmsg) -> Self {
        Self {
            sender: pm.user,
            target: pm.target,
            data: pm.data,
            tags: Arc::new(pm.tags),
        }
    }

    pub fn channel(&self) -> &str {
        &self.target
    }

    pub fn tags(&self) -> &Tags {
        &self.tags
    }
}
