use std::{future::Future, sync::Arc};

use super::{Command, Dispatch, IntoCallable, SharedCallable};
use crate::{
    help::Registry,
    prelude::{Message, State},
    render::Render,
};

pub struct Binding<'a, T> {
    this: Arc<T>,
    callables: Vec<SharedCallable>,
    state: &'a mut State,
}

impl<'a, T> IntoCallable for Binding<'a, T>
where
    T: Send + Sync + 'static,
{
    fn into_callable(self) -> SharedCallable {
        let callables = Arc::new(self.callables);
        let func = move |msg| {
            let callables = Arc::clone(&callables);
            async move { Dispatch::new(&callables).into_render(&msg).await.boxed() }
        };
        Arc::new(func)
    }
}

impl<'a, T> Binding<'a, T>
where
    T: Send + Sync + 'static,
{
    // TODO record descriptions
    pub fn create(state: &'a mut State, this: T) -> Self {
        Self {
            this: Arc::new(this),
            callables: Vec::new(),
            state,
        }
    }

    pub fn bind<F, Fut>(self, id: &'static str, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let reg: &Registry = self.state.get().expect("registry must exist");
        let cmd = reg.fetch(id);
        self.bind_cmd(cmd, func)
    }

    pub fn bind_cmd<F, Fut>(mut self, cmd: Command, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let cmd = Arc::new(cmd);
        let func = (cmd.clone(), {
            let cmd = cmd.clone();
            let this = self.this.clone();
            move |msg: Message| {
                super::wrap(msg, cmd.clone(), {
                    let this = this.clone();
                    move |msg| {
                        let this = this.clone();
                        async move { func(this, msg).await }
                    }
                })
            }
        });

        self.callables.push(Arc::new(func));
        self
    }

    pub fn listen<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let func = {
            let this = self.this.clone();
            move |msg| {
                let this = this.clone();
                async move { func(this, msg).await.boxed() }
            }
        };
        self.callables.push(Arc::new(func));
        self
    }
}
