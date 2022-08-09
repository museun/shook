use serenity::{
    model::prelude::{Channel, Message as DiscordMessage},
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

        let ch = match msg.channel(&ctx).await {
            Ok(Channel::Guild(ch)) => ch,
            _ => return,
        };

        log::debug!(target:"shook::discord", "[{}] {}: {}", ch.name(), msg.author.name, msg.content);

        let id = msg.channel_id;
        let sm = ShookMessage::new(
            super::Message::from_serenity(msg.clone()),
            MessageKind::Discord,
            self.state.clone(),
        );

        let ch = ch.name();
        for resp in dispatch_and_render(&self.callables, &sm, RenderFlavor::Discord).await {
            match resp {
                Response::Say(out) => {
                    log::trace!(target:"shook::discord","say [{ch}] {out}");
                    let _ = id.say(&ctx, out).await;
                }
                Response::Reply(out) => {
                    let sender = &msg.author.name;
                    log::trace!(target:"shook::discord","reply ({sender}) [{ch}] {out}");
                    let _ = msg.reply(&ctx, out).await;
                }
                Response::Problem(out) => {
                    log::trace!(target:"shook::discord","problem [{ch}] {out}");
                    let _ = id.say(&ctx, format!("a problem occurred: {out}")).await;
                }
            };
        }
    }
}
