use std::{future::Future, sync::Arc};

use super::{Command, Dispatch, IntoCallable, SharedCallable};
use crate::{help::Registry, prelude::Message, render::Render};

pub struct Group<'a> {
    callables: Vec<SharedCallable>,
    registry: &'a Registry,
}

impl<'a> IntoCallable for Group<'a> {
    fn into_callable(self) -> SharedCallable {
        let callables = Arc::new(self.callables);
        let func = move |msg| {
            let commands = Arc::clone(&callables);
            Box::pin(async move { Dispatch::new(&commands).into_render(&msg).await.boxed() })
        };
        Arc::new(func)
    }
}

impl<'a> Group<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self {
            callables: vec![],
            registry,
        }
    }

    pub fn bind<F, Fut>(self, id: &'static str, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let cmd = self.registry.fetch(id);
        self.bind_cmd(cmd, func)
    }

    pub fn bind_cmd<F, Fut>(mut self, cmd: Command, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let cmd = Arc::new(cmd);
        let func = {
            (cmd.clone(), move |msg: Message| {
                super::wrap(msg, cmd.clone(), func)
            })
        };

        self.callables.push(Arc::new(func));
        self
    }

    pub fn listen<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let func = move |msg| async move { func(msg).await.boxed() };
        self.callables.push(Arc::new(func));
        self
    }
}
