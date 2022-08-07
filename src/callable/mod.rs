use std::{future::Future, sync::Arc};

use crate::{
    args::{Arguments, Match},
    message::Message,
    render::Render,
    render::{BoxedRender, RenderFlavor, Response},
    BoxedFuture,
};

mod binding;
mod command;
mod dispatch;
mod group;

pub use binding::Binding;
pub use command::Command;
pub use dispatch::Dispatch;
pub use group::Group;

pub trait IntoCallable {
    fn into_callable(self) -> SharedCallable;
}

pub type SharedCallable = Arc<dyn CallableFn<Out = BoxedFuture<'static, BoxedRender>>>;

pub trait CallableFn
where
    Self: Send + Sync + 'static,
{
    type Out: Future + Send;
    fn call(&self, msg: Message) -> Self::Out;
    fn usage(&self) -> Option<&str> {
        None
    }
    fn description(&self) -> Option<&str> {
        None
    }
}

impl<F> CallableFn for Arc<F>
where
    F: CallableFn + ?Sized,
{
    type Out = F::Out;

    fn call(&self, msg: Message) -> Self::Out {
        (**self).call(msg)
    }
}

impl<F, Fut> CallableFn for F
where
    F: Fn(Message) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future + Send,
    Fut::Output: Render + Send,
{
    type Out = BoxedFuture<'static, Fut::Output>;

    fn call(&self, msg: Message) -> Self::Out {
        let this = self.clone();
        Box::pin(async move { (this)(msg).await })
    }
}

impl<F> CallableFn for (Command, F)
where
    F: CallableFn,
{
    type Out = F::Out;

    fn call(&self, msg: Message) -> Self::Out {
        let (_, this) = self;
        this.call(msg)
    }

    fn usage(&self) -> Option<&str> {
        let (Command { command, .. }, ..) = &self;
        Some(command)
    }

    fn description(&self) -> Option<&str> {
        let (Command { description, .. }, ..) = &self;
        description.as_deref()
    }
}

async fn wrap<F>(
    mut msg: Message,
    cmd: Command,
    func: impl Fn(Message) -> F + Send + Sync,
) -> BoxedRender
where
    F::Output: Render + Send,
    F: Future + Send,
{
    enum MatchError {
        Required { usage: String },
        NoMatch { usage: String },
    }

    impl Render for MatchError {
        fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
            // TODO make this better for discord
            let data = match self {
                Self::Required { usage } => format!("an argument is required: {usage}"),
                Self::NoMatch { usage } => format!("invalid arguments: {usage}"),
            };
            data.render(flavor)
        }
    }

    fn check_command(cmd: &Command, msg: &Message) -> bool {
        [&*cmd.command]
            .into_iter()
            .chain(cmd.aliases.iter().map(|c| &**c))
            .fold(false, |ok, c| ok ^ msg.match_command(c))
    }

    if !check_command(&cmd, &msg) {
        return ().boxed();
    }

    // TODO just do split_at
    let head = std::cmp::min(msg.command().len() + 1, msg.data().len());
    let input = &msg.data()[head..];

    if let Some(example) = &cmd.example {
        let args = match example.extract(input) {
            Match::Required => {
                let usage = cmd.command.to_string();
                return MatchError::Required { usage }.boxed();
            }

            Match::NoMatch => {
                let usage = cmd.command.to_string();
                return MatchError::NoMatch { usage }.boxed();
            }
            Match::Match(map) => Arguments { map },
            Match::Exact => Arguments::default(),
        };
        msg.get_args().replace(args);
    };

    func(msg).await.boxed()
}
