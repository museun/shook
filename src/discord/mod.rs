use serenity::{framework::StandardFramework, prelude::GatewayIntents, Client};

use crate::{callable::SharedCallable, state::GlobalState};

mod handler;
use handler::Handler;

mod message;
pub use message::Message;

pub async fn create_bot<const N: usize>(
    state: GlobalState,
    callables: [SharedCallable; N],
) -> anyhow::Result<()> {
    let token = std::env::var("SHAKEN_DISCORD_TOKEN").unwrap();

    log::info!("connecting to discord");
    let mut client = Client::builder(
        &token,
        GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler { state, callables })
    .framework(StandardFramework::new())
    .await?;

    log::info!("connected");
    log::info!("starting the discord bot");
    client.start().await.map_err(Into::into)
}
