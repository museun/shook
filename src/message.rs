use std::{any::Any, sync::Arc};

use serenity::model::prelude::ChannelId;

use crate::{
    args::Arguments,
    state::SharedState,
    twitch::{Privmsg, Tags},
};

pub struct TwitchMessage {
    sender: Arc<str>,
    target: Arc<str>,
    data: Arc<str>,
    tags: Arc<Tags>,
}

impl TwitchMessage {
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

pub struct DiscordMessage {
    sender: Arc<str>,
    target: ChannelId,
    data: Arc<str>,
}

impl DiscordMessage {
    pub fn from_serenity(msg: serenity::model::prelude::Message) -> Self {
        Self {
            sender: msg.author.name.into(),
            target: msg.channel_id,
            data: msg.content.into(),
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        self.target
    }
}

#[rustfmt::skip]
macro_rules! message {
($($ident:ident)*) => {
    $(
        impl MessageType for $ident {
            fn data(&self) -> &str { &self.data }
            fn sender_name(&self) -> &str { &self.sender }
            fn as_any(&self) -> &dyn Any { self }
        }
    )*
};
}

message! {
    TwitchMessage
    DiscordMessage
}

pub trait MessageType
where
    Self: Any + Send + Sync + 'static,
{
    fn data(&self) -> &str;
    fn sender_name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone)]
pub struct Message {
    inner: Arc<dyn MessageType>,
    kind: MessageKind,
    state: SharedState,
    pub(super) args: Option<Arguments>,
}

impl Message {
    pub fn new(inner: impl MessageType, kind: MessageKind, state: SharedState) -> Self {
        Self {
            inner: Arc::new(inner),
            kind,
            state,
            args: None,
        }
    }

    pub fn data(&self) -> &str {
        self.inner.data()
    }

    pub fn sender_name(&self) -> &str {
        self.inner.sender_name()
    }

    pub fn match_command(&self, right: &str) -> bool {
        self.command() == Self::split_command(right)
    }

    pub fn command(&self) -> &str {
        Self::split_command(self.data())
    }

    pub fn args(&self) -> &Arguments {
        self.args.as_ref().unwrap()
    }

    pub fn state(&self) -> &SharedState {
        &self.state
    }

    pub const fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn as_twitch(&self) -> Option<&TwitchMessage> {
        self.inner.as_any().downcast_ref()
    }

    pub fn as_discord(&self) -> Option<&DiscordMessage> {
        self.inner.as_any().downcast_ref()
    }

    fn split_command(input: &str) -> &str {
        input
            .split_once(' ')
            .map(|(k, _)| k)
            .unwrap_or_else(|| input)
    }
}

#[derive(Copy, Clone)]
pub enum MessageKind {
    Twitch,
    Discord,
}
