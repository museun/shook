use std::sync::Arc;

use crate::{args::Arguments, state::GlobalState};

pub trait MessageType
where
    Self: Send + Sync + 'static,
{
    fn data(&self) -> &str;
    fn sender_name(&self) -> &str;
    fn source(&self) -> &str;
    fn is_from_admin(&self) -> bool {
        false
    }
    fn is_from_moderator(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Message {
    inner: Arc<dyn MessageType>,
    state: GlobalState,
    args: Option<Arguments>,
}

impl Message {
    pub fn new(inner: impl MessageType, state: GlobalState) -> Self {
        Self {
            inner: Arc::new(inner),
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

    pub fn source(&self) -> &str {
        self.inner.source()
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

    pub async fn streamer_name(&self) -> String {
        self.state
            .get_owned::<crate::prelude::StreamerName>()
            .await
            .0
    }

    pub async fn is_from_owner(&self) -> bool {
        self.sender_name() == self.streamer_name().await
    }

    pub fn require_moderator(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.is_from_moderator(),
            "that requires you to be a moderator"
        );
        Ok(())
    }

    pub fn require_broadcaster(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.is_from_broadcaster(),
            "that requires you to be the broadcaster"
        );
        Ok(())
    }

    pub fn require_elevation(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.is_from_moderator() || self.is_from_broadcaster(),
            "that requires you to be a moderator or the broadcaster"
        );
        Ok(())
    }

    pub fn is_from_broadcaster(&self) -> bool {
        self.inner.is_from_admin()
    }

    pub fn is_from_moderator(&self) -> bool {
        self.inner.is_from_moderator()
    }

    pub(super) fn get_args(&mut self) -> &mut Option<Arguments> {
        &mut self.args
    }

    fn split_command(input: &str) -> &str {
        input.split_once(' ').map_or_else(|| input, |(k, _)| k)
    }
}
