use std::{future::Future, pin::Pin};

pub mod help;

pub mod args;
pub mod callable;
pub mod message;
pub mod render;

mod format;
pub use format::{FormatTime, IterExt};

pub mod config;
pub use config::{ConfigPath, PersistFromConfig};

mod state;

// #[cfg(test)]
pub mod testing;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

// TODO DontCare

#[macro_use]
pub mod prelude {
    pub use std::sync::Arc;

    #[derive(Clone, Debug)]
    pub struct StreamerName(pub String);

    pub use crate::callable::{self, Binding, Command, IntoCallable, SharedCallable};
    pub type SharedRegistry = Arc<crate::help::Registry>;
    pub use crate::message::Message;
    pub use crate::render::{Render, RenderFlavor, Response, Simple};
    pub use crate::state::{GlobalState, State};
}

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
