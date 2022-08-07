use std::{future::Future, sync::Arc};

use crate::{prelude::Message, render::Render};

use super::{Command, Dispatch, IntoCallable, SharedCallable};

#[derive(Default)]
pub struct Group(Vec<SharedCallable>);

impl IntoCallable for Group {
    fn into_callable(self) -> SharedCallable {
        let callables = Arc::new(self.0);
        let func = move |msg| {
            let callables = Arc::clone(&callables);
            async move { Dispatch::new(&callables).into_render(&msg).await.boxed() }
        };
        Arc::new(func)
    }
}

impl Group {
    pub fn bind<F, Fut>(mut self, cmd: Command, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let func = Arc::new({
            let cmd = cmd.clone();
            move |msg: Message| super::wrap(msg, cmd.clone(), func.clone())
        });

        self.0.push(Arc::new((cmd, func)));
        self
    }

    pub fn listen<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let func = Arc::new(move |msg| {
            let func = func.clone();
            async move { func(msg).await.boxed() }
        });
        self.0.push(func);
        self
    }
}
