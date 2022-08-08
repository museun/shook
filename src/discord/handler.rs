use serenity::{
    model::prelude::Message as DiscordMessage,
    prelude::{Context, EventHandler},
};

use crate::{
    callable::SharedCallable,
    message::Message as ShookMessage,
    message::MessageKind,
    render::{dispatch_and_render, RenderFlavor, Response},
    state::GlobalState,
};

pub struct Handler<const N: usize> {
    pub state: GlobalState,
    pub callables: [SharedCallable; N],
}

#[async_trait::async_trait]
impl<const N: usize> EventHandler for Handler<N> {
    async fn message(&self, ctx: Context, msg: DiscordMessage) {
        if msg.author.bot || msg.is_private() {
            return;
        }

        let id = msg.channel_id;
        let sm = ShookMessage::new(
            super::Message::from_serenity(msg),
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
