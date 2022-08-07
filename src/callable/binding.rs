use std::{future::Future, sync::Arc};

use super::{Command, Dispatch, IntoCallable, SharedCallable};
use crate::{prelude::Message, render::Render};

pub struct Binding<T> {
    this: Arc<T>,
    callables: Vec<SharedCallable>,
}

impl<T> IntoCallable for Binding<T>
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

impl<T> Binding<T>
where
    T: Send + Sync + 'static,
{
    // TODO record descriptions
    pub fn create(this: T) -> Self {
        Self {
            this: Arc::new(this),
            callables: Vec::new(),
        }
    }

    pub fn bind<F, Fut>(mut self, cmd: Command, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send,
    {
        let func = Arc::new({
            let cmd = cmd.clone();
            let this = self.this.clone();
            move |msg: Message| {
                super::wrap(msg, cmd.clone(), {
                    let func = func.clone();
                    let this = this.clone();
                    move |msg| {
                        let func = func.clone();
                        let this = this.clone();
                        async move { func(this, msg).await }
                    }
                })
            }
        });

        self.callables.push(Arc::new((cmd, func)));
        self
    }

    pub fn listen<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send,
    {
        let func = Arc::new({
            let this = self.this.clone();
            move |msg| {
                let func = func.clone();
                let this = this.clone();
                async move { func(this, msg).await.boxed() }
            }
        });
        self.callables.push(func);
        self
    }
}
