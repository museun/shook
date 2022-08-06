use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{
    args::{Arguments, ExampleArgs, Match},
    message::Message,
    render::{BoxedRender, Response},
    state::SharedState,
    BoxedFuture, Render,
};

use tokio_stream::{Stream, StreamExt};

#[async_trait::async_trait]
pub trait Bind
where
    Self: Sized + Send + Sync + 'static,
{
    async fn bind(state: SharedState) -> anyhow::Result<Binding<Self>>;
}

#[derive(Clone)]
pub struct Command {
    pub command: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

impl<K, V> From<(K, V)> for Command
where
    K: Into<Cow<'static, str>>,
    V: Into<Cow<'static, str>>,
{
    fn from((command, description): (K, V)) -> Self {
        Command {
            command: command.into(),
            description: description.into(),
        }
    }
}

pub struct Dispatch<'a, T, F>
where
    F: Fn(&'a T) -> &Arc<Callable> + Copy,
{
    pub seq: &'a [T],
    pub extract: F,
}

impl<'a, T, F> Dispatch<'a, T, F>
where
    F: Fn(&'a T) -> &Arc<Callable> + Copy,
{
    pub const fn new(seq: &'a [T], extract: F) -> Self {
        Self { seq, extract }
    }

    pub async fn dispatch(self, msg: &Message) -> impl Stream<Item = BoxedRender> {
        let (tx, rx) = tokio::sync::mpsc::channel(std::cmp::max(1, self.seq.len()));
        for callable in self.seq.iter().map(self.extract).map(Arc::clone) {
            let tx = tx.clone();
            let msg = msg.clone();
            let _ = tokio::spawn(async move {
                let render = callable(msg).await;
                let _ = tx.send(render).await;
            });
        }
        drop(tx);
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

pub type Callable = dyn Fn(Message) -> BoxedFuture<'static, BoxedRender> + Send + Sync + 'static;

pub fn listen<F, Fut, R>(func: F) -> Arc<Callable>
where
    F: Fn(Message) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: Render,
{
    let func = move |msg: Message| {
        let func = func.clone();
        Box::pin(async move { Box::new(func(msg).await) as BoxedRender })
            as BoxedFuture<'static, BoxedRender>
    };

    Arc::new(func)
}

pub fn bind<F, Fut, R>(cmd: impl Into<Command>, func: F) -> anyhow::Result<Arc<Callable>>
where
    F: Fn(Message) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: Render,
{
    let cmd = cmd.into();

    let func = {
        let cmd = cmd.clone();
        let example = ExampleArgs::parse(&*cmd.command).map(Arc::new)?;
        move |mut msg: Message| {
            let func = func.clone();
            let example = Arc::clone(&example);
            let cmd = cmd.clone();

            Box::pin(async move {
                if !msg.match_command(&cmd.command) {
                    return ().boxed();
                }

                let head = std::cmp::min(msg.command().len() + 1, msg.data().len());
                let input = &msg.data()[head..];

                let args = match example.extract(input) {
                    Match::Required => {
                        return MatchError::Required {
                            usage: cmd.command.to_string(),
                        }
                        .boxed();
                    }

                    Match::NoMatch => {
                        return MatchError::NoMatch {
                            usage: cmd.command.to_string(),
                        }
                        .boxed();
                    }
                    Match::Match(map) => Arguments { map },
                    Match::Exact => Arguments::default(),
                };

                msg.args.replace(args);

                Box::new(func(msg).await) as BoxedRender
            }) as BoxedFuture<'static, BoxedRender>
        }
    };

    Ok(Arc::new(func))
}

pub struct Binding<T> {
    this: Arc<T>,
    commands: Vec<(Command, Arc<Callable>)>,
    passives: Vec<Arc<Callable>>,
}

impl<T> Binding<T>
where
    T: Send + Sync + 'static,
{
    pub fn create(this: T) -> Self {
        Self {
            this: Arc::new(this),
            commands: Vec::new(),
            passives: Vec::new(),
        }
    }

    pub fn bind<F, Fut, R>(mut self, cmd: impl Into<Command>, func: F) -> anyhow::Result<Self>
    where
        F: Fn(Arc<T>, Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = R> + Send,
        R: Render,
    {
        let cmd = cmd.into();
        let func = {
            let cmd = cmd.clone();
            let example = ExampleArgs::parse(&*cmd.command).map(Arc::new)?;
            let this = Arc::clone(&self.this);
            move |mut msg: Message| {
                let this = Arc::clone(&this);
                let func = func.clone();
                let example = Arc::clone(&example);
                let cmd = cmd.clone();

                Box::pin(async move {
                    if !msg.match_command(&cmd.command) {
                        return ().boxed();
                    }

                    let head = std::cmp::min(msg.command().len() + 1, msg.data().len());
                    let input = &msg.data()[head..];

                    let args = match example.extract(input) {
                        Match::Required => {
                            return MatchError::Required {
                                usage: cmd.command.to_string(),
                            }
                            .boxed();
                        }

                        Match::NoMatch => {
                            return MatchError::NoMatch {
                                usage: cmd.command.to_string(),
                            }
                            .boxed();
                        }
                        Match::Match(map) => Arguments { map },
                        Match::Exact => Arguments::default(),
                    };

                    msg.args.replace(args);

                    Box::new(func(this, msg).await) as BoxedRender
                }) as BoxedFuture<'static, BoxedRender>
            }
        };

        self.commands.push((cmd, Arc::new(func)));
        Ok(self)
    }

    fn listen<F, Fut, R>(mut self, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = R> + Send,
        R: Render,
    {
        let this = Arc::clone(&self.this);
        let func = move |msg| {
            let this = Arc::clone(&this);
            let func = func.clone();
            Box::pin(async move { Box::new(func(this, msg).await) as BoxedRender })
                as BoxedFuture<'static, BoxedRender>
        };
        self.passives.push(Arc::new(func));
        self
    }

    pub async fn dispatch(&self, msg: &Message) -> impl Stream<Item = BoxedRender> + '_ {
        let left = Dispatch::new(&*self.commands, |(_, cmd)| cmd);
        let right = Dispatch::new(&*self.passives, std::convert::identity);

        let (left, right) = tokio::join!(
            left.dispatch(msg), //
            right.dispatch(msg),
        );

        left.chain(right)
    }

    pub fn into_callable(self) -> Arc<Callable> {
        let this = Arc::new(self);
        Arc::new(move |msg| {
            let this = Arc::clone(&this);
            Box::pin(async move {
                let mut stream = this.dispatch(&msg).await;
                let mut out = vec![];
                while let Some(resp) = stream.next().await {
                    out.push(resp);
                }
                out.boxed()
            })
        })
    }
}

enum MatchError {
    Required { usage: String },
    NoMatch { usage: String },
}

impl Render for MatchError {
    fn render_twitch(&self) -> Vec<Response> {
        let data = match self {
            Self::Required { usage } => format!("an argument is required: {usage}"),
            Self::NoMatch { usage } => format!("invalid arguments: {usage}"),
        };
        data.render_twitch()
    }
}
