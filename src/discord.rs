use std::sync::Arc;

use serenity::{
    framework::StandardFramework,
    model::prelude::Message,
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};
use tokio_stream::StreamExt;

use crate::{
    binding::{Callable, Dispatch},
    message::Message as ShookMessage,
    message::{DiscordMessage, MessageKind},
    render::Response,
    state::SharedState,
};

pub async fn create_bot<const N: usize>(
    state: SharedState,
    callables: [Arc<Callable>; N],
) -> anyhow::Result<()> {
    let token = std::env::var("SHAKEN_DISCORD_TOKEN").unwrap();

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

    client.start().await.map_err(Into::into)
}

struct Handler<const N: usize> {
    state: SharedState,
    callables: [Arc<Callable>; N],
}

#[async_trait::async_trait]
impl<const N: usize> EventHandler for Handler<N> {
    async fn message(&self, ctx: Context, msg: Message) {
        let sm = ShookMessage::new(
            DiscordMessage::from_serenity(msg),
            MessageKind::Discord,
            self.state.clone(),
        );

        let mut stream = Dispatch::new(&self.callables, std::convert::identity)
            .dispatch(&sm)
            .await;

        let id = sm.as_discord().unwrap().channel_id();

        while let Some(resp) = stream.next().await {
            for resp in resp.render_discord() {
                let out = match resp {
                    Response::Say(msg) => {
                        let _ = id.say(&ctx, msg).await;
                    }
                    Response::Reply(msg) => {
                        let _ = id.say(&ctx, msg).await;
                    }
                    Response::Problem(msg) => {
                        let _ = id.say(&ctx, format!("a problem occurred: {msg}")).await;
                    }
                };
            }
        }
    }
}
