use std::{future::Future, sync::Arc};

use super::{Command, Dispatch, IntoCallable, SharedCallable};
use crate::{prelude::Message, render::Render};

pub struct Group {
    commands: Arc<Vec<SharedCallable>>,
}

impl IntoCallable for Group {
    fn into_callable(self) -> SharedCallable {
        Arc::new({
            move |msg| {
                let commands = Arc::clone(&self.commands);
                Box::pin(async move { Dispatch::new(&commands).into_render(&msg).await.boxed() })
            }
        })
    }
}

impl Group {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(vec![]),
        }
    }

    pub fn bind<F, Fut>(mut self, cmd: Command, func: F) -> Self
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

        Arc::get_mut(&mut self.commands)
            .expect("single ownership at this point")
            .push(Arc::new(func));
        self
    }

    pub fn listen<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Message) -> Fut + Copy + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Render + Send + 'static,
    {
        let func = move |msg| async move { func(msg).await.boxed() };

        Arc::get_mut(&mut self.commands)
            .expect("single ownership at this point")
            .push(Arc::new(func));
        self
    }
}
