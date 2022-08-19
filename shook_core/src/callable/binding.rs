use std::{future::Future, sync::Arc};

use super::{Command, Dispatch, IntoCallable, SharedCallable};
use crate::{
    prelude::{Message, SharedRegistry},
    render::Render,
    state::GlobalState,
};

fn command_name<A, B, C>(f: impl Fn(A, B) -> C + Copy) -> String {
    fn ty<T>(_d: &T) -> &'static str {
        std::any::type_name::<T>()
    }
    use heck::ToSnekCase as _;

    let mut v = ty(&f)
        .rsplitn(3, "::")
        .take(2)
        .map(|s| s.to_snek_case())
        .collect::<Vec<_>>();
    v.reverse();
    v.join("::")
}

pub struct Binding<T> {
    this: Arc<T>,
    callables: Vec<SharedCallable>,
    registry: SharedRegistry,
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
    pub async fn create(state: GlobalState, this: T) -> Self {
        Self {
            this: Arc::new(this),
            callables: Vec::new(),
            registry: state.get_owned().await,
        }
    }

    pub fn bind<F, Fut>(self, func: F) -> Self
    where
        F: Fn(Arc<T>, Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let id = command_name(func);
        let cmd = self.registry.fetch(&id);
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
