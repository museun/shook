use std::{borrow::Cow, future::Future, sync::Arc};

use persist::{tokio::PersistExt, yaml::Yaml};

use crate::{
    callable::CallableFn,
    discord::Message as DiscordMessage,
    help::Registry,
    message::MessageKind,
    prelude::{GlobalState, Message, SharedCallable, State},
    render::{BoxedRender, Render, RenderFlavor, Response},
    twitch::{Message as TwitchMessage, Privmsg, Tags},
    BoxedFuture,
};

#[async_trait::async_trait]
pub trait Mock
where
    Self: Sized + Send + Sync + 'static,
{
    async fn mock(self) -> TestBinding;
    async fn mock_with_state(self, state: State) -> TestBinding;
}

#[async_trait::async_trait]
impl<F, Fut, C> Mock for F
where
    F: Fn(GlobalState) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<C>> + Send,
    C: CallableFn<Out = BoxedFuture<'static, BoxedRender>>,
{
    async fn mock(self) -> TestBinding {
        Self::mock_with_state(self, <_>::default()).await
    }

    async fn mock_with_state(self, mut state: State) -> TestBinding {
        let registry = Registry::load_from_file::<Yaml>("default_help")
            .await
            .unwrap();
        state.insert(registry);

        let state = GlobalState::new(state);
        let callable = (self)(state.clone()).await.expect("valid binding");

        TestBinding {
            callable: Arc::new(callable),
            responses: Vec::new(),
            channel: Cow::from("#test_chanenl"),
            sender: Cow::from("test_user"),
            state,
            mod_: false,
            admin: false,
            tags: Tags::default(),
        }
    }
}

pub struct TestBinding {
    callable: SharedCallable,
    responses: Vec<Response>,
    channel: Cow<'static, str>,
    sender: Cow<'static, str>,
    state: GlobalState,
    mod_: bool,
    admin: bool,
    tags: Tags,
}

impl TestBinding {
    fn insert_badge(&mut self, key: &str, val: &str) {
        use std::fmt::Write as _;

        use std::collections::hash_map::Entry::*;
        match self.tags.map.entry(Box::from("badges")) {
            Occupied(mut e) => {
                let mut s = e.get().to_string();
                let _ = write!(&mut s, ",{key}/{val}");
                *e.get_mut() = s.into_boxed_str();
            }
            Vacant(e) => {
                e.insert(format!("{key}/{val}").into_boxed_str());
            }
        }
    }

    pub fn with_moderator(mut self) -> Self {
        self.insert_badge("moderator", "1");
        self.mod_ = true;
        self
    }

    pub fn with_admin(mut self) -> Self {
        self.insert_badge("broadcaster", "1");
        self.admin = true;
        self
    }

    pub fn with_sender(mut self, sender: &str) -> Self {
        self.sender = sender.to_string().into();
        self
    }

    pub fn with_channel(mut self, channel: &str) -> Self {
        self.channel = channel.to_string().into();
        self
    }

    pub fn get_response(&mut self) -> Vec<Response> {
        std::mem::take(&mut self.responses)
    }

    pub async fn send_twitch_message(&mut self, data: &str) {
        let msg = Message::new(
            TwitchMessage::from_pm(Privmsg {
                tags: self.tags.clone(),
                user: self.sender.clone().into(),
                target: self.channel.clone().into(),
                data: data.into(),
            }),
            MessageKind::Twitch,
            self.state.clone(),
        );
        self.send_message(msg).await
    }

    pub async fn send_discord_message(&mut self, data: &str) {
        let msg = Message::new(
            DiscordMessage::mock(&self.sender, 42, data),
            MessageKind::Discord,
            self.state.clone(),
        );
        self.send_message(msg).await
    }

    async fn send_message(&mut self, msg: Message) {
        let flavor = match msg.kind() {
            MessageKind::Twitch => RenderFlavor::Twitch,
            MessageKind::Discord => RenderFlavor::Discord,
        };
        let out = self.callable.call(msg).await.render(flavor);
        self.responses.extend(out);
    }
}
