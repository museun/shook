use std::{future::Future, pin::Pin};

#[macro_use]
pub mod help;

pub mod args;
pub mod callable;
pub mod message;
pub mod render;

mod format;
pub use format::{FormatTime, IterExt};

mod state;

// #[cfg(test)]
// pub mod testing;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

// TODO DontCare

#[macro_use]
pub mod prelude {
    pub use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub struct Streamer(pub String);

    pub use crate::cmd;

    pub use crate::callable::{self, Binding, Command, Group, IntoCallable, SharedCallable};
    pub use crate::help::Registry;
    pub use crate::message::Message;
    pub use crate::render::{
        Bold, Code, Hidden, Italics, Render, RenderFlavor, Response, Simple, Strikeout, Underline,
    };
    pub use crate::state::{GlobalState, State};
}

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
