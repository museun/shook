use std::{borrow::Cow, future::Future, sync::Arc};

use persist::{tokio::PersistExt, yaml::Yaml};

use crate::{
    callable::CallableFn,
    message::MessageType,
    prelude::{GlobalState, Message, Render, Response, SharedCallable, State},
    render::{BoxedRender, RenderFlavor},
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
        let registry = crate::help::Registry::load_from_file::<Yaml>(concat!(
            // this points to the crate in the workspace, not the workspace itself
            env!("CARGO_MANIFEST_DIR"),
            "/../default_help"
        ))
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
            moderator: false,
            admin: false,
        }
    }
}

pub struct TestBinding {
    callable: SharedCallable,
    responses: Vec<Response>,
    channel: Cow<'static, str>,
    sender: Cow<'static, str>,
    state: GlobalState,
    moderator: bool,
    admin: bool,
}

impl TestBinding {
    pub fn with_moderator(mut self) -> Self {
        self.moderator = true;
        self
    }

    pub fn with_admin(mut self) -> Self {
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

    pub async fn send_message<B: BuildTestMessage + Send>(&mut self, data: &str) {
        let mut builder = B::default()
            .with_data(data)
            .with_channel(&self.channel)
            .with_sender(&self.sender);

        if self.admin {
            builder = builder.with_admin()
        }
        if self.moderator {
            builder = builder.with_moderator()
        }

        let flavor = builder.render_flavor();
        let msg = Message::new(builder.into_message(), self.state.clone());
        let out = self.callable.call(msg).await.render(flavor);
        self.responses.extend(out);
    }
}

pub trait BuildTestMessage
where
    Self: Sized + Default,
{
    type Output: MessageType;
    fn into_message(self) -> Self::Output;

    fn with_admin(self) -> Self {
        self
    }

    fn with_moderator(self) -> Self {
        self
    }

    fn with_sender(self, _: &str) -> Self {
        self
    }

    fn with_channel(self, _: &str) -> Self {
        self
    }

    fn with_data(self, _: &str) -> Self {
        self
    }

    fn with_flavor(self, _: RenderFlavor) -> Self {
        self
    }

    fn render_flavor(&self) -> RenderFlavor {
        RenderFlavor::Twitch
    }
}

pub struct MockMessage {
    source: String,
    sender: String,
    data: String,
    admin: bool,
    moderator: bool,
    flavor: RenderFlavor,
}

impl Default for MockMessage {
    fn default() -> Self {
        Self {
            source: Default::default(),
            sender: Default::default(),
            data: Default::default(),
            admin: Default::default(),
            moderator: Default::default(),
            flavor: RenderFlavor::Twitch,
        }
    }
}

impl BuildTestMessage for MockMessage {
    type Output = Self;

    fn into_message(self) -> Self::Output {
        self
    }

    fn with_admin(mut self) -> Self {
        self.admin = !self.admin;
        self
    }

    fn with_moderator(mut self) -> Self {
        self.moderator = !self.moderator;
        self
    }

    fn with_sender(mut self, sender: &str) -> Self {
        self.sender = sender.into();
        self
    }

    fn with_channel(mut self, channel: &str) -> Self {
        self.source = channel.into();
        self
    }

    fn with_data(mut self, data: &str) -> Self {
        self.data = data.into();
        self
    }

    fn with_flavor(mut self, flavor: RenderFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    fn render_flavor(&self) -> RenderFlavor {
        self.flavor
    }
}

impl MessageType for MockMessage {
    fn data(&self) -> &str {
        &self.data
    }

    fn sender_name(&self) -> &str {
        &self.sender
    }

    fn source(&self) -> &str {
        &self.source
    }
}
