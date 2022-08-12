use shook_core::message::MessageType;
use twilight_model::channel::Message;

pub struct TwilightMessage {
    pub inner: Message,
    pub source: String,
}

impl std::ops::Deref for TwilightMessage {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MessageType for TwilightMessage {
    fn data(&self) -> &str {
        &self.content
    }

    fn sender_name(&self) -> &str {
        &self.author.name
    }

    fn source(&self) -> &str {
        &self.source
    }
}
