use std::sync::Arc;

use serenity::{
    framework::StandardFramework,
    model::prelude::{ChannelId, Message as DiscordMessage},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

use crate::{
    callable::SharedCallable,
    message::Message as ShookMessage,
    message::MessageKind,
    render::{dispatch_and_render, RenderFlavor, Response},
    state::GlobalState,
};

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

struct Handler<const N: usize> {
    state: GlobalState,
    callables: [SharedCallable; N],
}

#[async_trait::async_trait]
impl<const N: usize> EventHandler for Handler<N> {
    async fn message(&self, ctx: Context, msg: DiscordMessage) {
        let id = msg.channel_id;

        let sm = ShookMessage::new(
            Message::from_serenity(msg),
            MessageKind::Discord,
            self.state.clone(),
        );

        for resp in dispatch_and_render(&self.callables, &sm, RenderFlavor::Discord).await {
            match resp {
                Response::Say(msg) => {
                    let _ = id.say(&ctx, msg).await;
                }
                Response::Reply(msg) => {
                    // TODO reply
                    let _ = id.say(&ctx, msg).await;
                }
                Response::Problem(msg) => {
                    let _ = id.say(&ctx, format!("a problem occurred: {msg}")).await;
                }
            };
        }
    }
}

pub struct Message {
    pub(crate) sender: Arc<str>,
    target: ChannelId,
    pub(crate) data: Arc<str>,
}

impl Message {
    pub fn from_serenity(msg: serenity::model::prelude::Message) -> Self {
        Self {
            sender: msg.author.name.into(),
            target: msg.channel_id,
            data: msg.content.into(),
        }
    }

    #[cfg(test)]
    pub fn mock(sender: &str, target: u64, data: &str) -> Self {
        Self {
            sender: sender.into(),
            target: ChannelId(target),
            data: data.into(),
        }
    }

    pub const fn channel_id(&self) -> ChannelId {
        self.target
    }
}
