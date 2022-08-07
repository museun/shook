use std::{any::Any, sync::Arc};

use crate::{args::Arguments, state::GlobalState};

#[rustfmt::skip]
macro_rules! message {
($($ident:path)*) => {
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
    crate::twitch::Message
    crate::discord::Message
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
    state: GlobalState,
    args: Option<Arguments>,
}

impl Message {
    pub fn new(inner: impl MessageType, kind: MessageKind, state: GlobalState) -> Self {
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

    pub const fn state(&self) -> &GlobalState {
        &self.state
    }

    pub const fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn as_twitch(&self) -> Option<&crate::twitch::Message> {
        self.inner.as_any().downcast_ref()
    }

    pub fn as_discord(&self) -> Option<&crate::discord::Message> {
        self.inner.as_any().downcast_ref()
    }

    pub(super) fn get_args(&mut self) -> &mut Option<Arguments> {
        &mut self.args
    }

    fn split_command(input: &str) -> &str {
        input
            .split_once(' ')
            .map(|(k, _)| k)
            .unwrap_or_else(|| input)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[non_exhaustive]
pub enum MessageKind {
    Twitch,
    Discord,
}
