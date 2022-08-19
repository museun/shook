use std::{future::Future, sync::Arc};

use crate::{message::Message, render::BoxedRender, render::Render, BoxedFuture};

mod binding;
mod command;
mod dispatch;

pub use binding::Binding;
pub use command::Command;
pub use dispatch::Dispatch;

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

    fn all_commands(&self) -> Vec<&Command> {
        vec![]
    }
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

    #[inline]
    fn all_commands(&self) -> Vec<&Command> {
        (**self).all_commands()
    }

    #[inline]
    fn usage(&self) -> Option<&str> {
        (**self).usage()
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        (**self).description()
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

impl<F> CallableFn for (Arc<Command>, F)
where
    F: CallableFn,
{
    type Out = F::Out;

    fn call(&self, msg: Message) -> Self::Out {
        let (_, this) = self;
        this.call(msg)
    }

    fn usage(&self) -> Option<&str> {
        let cmd = &*self.0;
        Some(&cmd.command)
    }

    fn description(&self) -> Option<&str> {
        let cmd = &*self.0;
        cmd.description.as_deref()
    }

    // TODO this should be a different return type
    fn all_commands(&self) -> Vec<&Command> {
        vec![&self.0]
    }
}

mod wrap;
pub(self) use wrap::wrap;
