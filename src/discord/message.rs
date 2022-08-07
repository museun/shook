use std::sync::Arc;

use serenity::model::prelude::ChannelId;

#[derive(Debug)]
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
