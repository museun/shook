use std::{future::Future, pin::Pin};

pub mod args;
pub mod callable;
pub mod config;
pub mod discord;
pub mod helix;
pub mod help;
pub mod message;
pub mod persist;
pub mod queue;
pub mod render;
pub mod twitch;

mod format;
pub use format::{FormatTime, IterExt};

mod serde;
mod state;

// #[cfg(test)]
pub mod testing;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

// TODO DontCare

#[macro_use]
pub mod prelude {
    pub use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub struct Streamer(pub String);

    pub use crate::callable::{self, Binding, Command, Group, IntoCallable, SharedCallable};
    pub use crate::help::Registry;
    pub use crate::message::Message;
    pub use crate::render::{
        Bold, Code, Hidden, Italics, Render, RenderFlavor, Response, Simple, Strikeout, Underline,
    };
    pub use crate::state::{GlobalState, State};
}

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
