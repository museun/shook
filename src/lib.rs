use std::{future::Future, pin::Pin};

pub mod args;
pub mod callable;

mod state;

pub mod discord;
pub mod message;
pub mod render;
pub mod twitch;

mod format;
pub use format::FormatTime;

pub mod help;
pub mod persist;

#[cfg(test)]
pub mod testing;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

// TODO DontCare

#[macro_use]
pub mod prelude {
    pub fn cmd(cmd: &str) -> Command {
        Command::new(cmd)
    }

    pub use crate::callable::{self, Binding, Command, Group, IntoCallable, SharedCallable};
    pub use crate::message::Message;
    pub use crate::render::{Render, Response};
    pub use crate::state::GlobalState;

    pub use std::sync::Arc;
}
