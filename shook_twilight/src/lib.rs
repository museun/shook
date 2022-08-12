use std::sync::Arc;

use anyhow::Context;

use tokio_stream::StreamExt as _;
use twilight_gateway::{Intents, Shard};
use twilight_http::{request::channel::message::create_message::CreateMessage, Client};
use twilight_model::{
    channel::message::MessageType,
    channel::Message,
    id::{marker::ChannelMarker, Id},
};

use shook_core::{
    callable::SharedCallable,
    prelude::GlobalState,
    render::{dispatch_and_render, RenderFlavor},
};

pub mod config;
mod state;

pub async fn create_bot<const N: usize>(
    state: GlobalState,
    handlers: [SharedCallable; N],
) -> anyhow::Result<()> {
    let config: crate::config::Discord = state.get_owned().await;

    let client = Arc::new(twilight_http::Client::new(
        config.oauth_token.clone().into_inner(),
    ));

    let (shard, mut events) = Shard::new(
        config.oauth_token.into_inner(),
        Intents::GUILDS | Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    );
    shard.start().await?;

    let bot = Bot {
        state,
        handlers,
        client: client.clone(),
    };

    let seen = state::DiscordState::default();
    let mut our_user_id = None;

    while let Some(event) = events.next().await {
        match event {
            twilight_gateway::Event::MessageCreate(msg)
                if matches!(msg.kind, MessageType::Regular)
                    && Some(msg.author.id) != our_user_id =>
            {
                let channel = seen
                    .channels
                    .update(msg.channel_id, {
                        let client = client.clone();
                        let id = msg.channel_id;
                        move || {
                            let client = client.clone();
                            async move { get_channel_name(&client, id).await }
                        }
                    })
                    .await?;

                log::debug!(target: "shook::discord", "[{}] {}: {}", channel, msg.author.name, msg.content);
                bot.handle(msg.0).await?;
            }
            twilight_gateway::Event::Ready(msg) => {
                log::debug!("discord bot name: {}, id: {}", msg.user.name, msg.user.id);
                our_user_id.get_or_insert(msg.user.id);
            }
            _ => {}
        }
    }

    Ok(())
}

struct Bot<const N: usize> {
    state: GlobalState,
    handlers: [SharedCallable; N],
    client: Arc<Client>,
}

impl<const N: usize> Bot<N> {
    async fn handle(&self, msg: Message) -> anyhow::Result<()> {
        use shook_core::prelude::{Message as ShookMessage, Response};

        let (ch, id) = (msg.channel_id, msg.id);

        let source = get_channel_name(&self.client, ch).await?;

        let msg = ShookMessage::new(TwilightMessage { inner: msg, source }, self.state.clone());
        for resp in dispatch_and_render(&self.handlers, &msg, RenderFlavor::Discord).await {
            match resp {
                Response::Say(resp) => {
                    self.create_message(ch, &resp, |msg| msg).await?;
                }
                Response::Reply(resp) => {
                    self.create_message(ch, &resp, |msg| msg).await?;
                }
                Response::Problem(resp) => {
                    let resp = format!("I ran into a problem: {resp}");
                    self.create_message(ch, &resp, |msg| msg.reply(id)).await?;
                }
            }
        }

        Ok(())
    }

    async fn create_message<'r>(
        &'r self,
        ch: Id<ChannelMarker>,
        data: &'r str,
        map: impl Fn(CreateMessage<'r>) -> CreateMessage<'r>,
    ) -> anyhow::Result<()> {
        let msg = self.client.create_message(ch).content(data).map(map)?;
        let _ = msg.exec().await;
        Ok(())
    }
}

struct TwilightMessage {
    inner: Message,
    source: String,
}

impl std::ops::Deref for TwilightMessage {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl shook_core::message::MessageType for TwilightMessage {
    fn data(&self) -> &str {
        &self.content
    }

    fn sender_name(&self) -> &str {
        &self.author.name
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

async fn get_channel_name(client: &Client, id: Id<ChannelMarker>) -> anyhow::Result<String> {
    let resp = client.channel(id).exec().await?;

    let name = resp
        .model()
        .await?
        .name
        .with_context(|| "cannot find name for {id}")?;

    Ok(name)
}
