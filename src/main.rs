#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use std::{future::Future, pin::Pin};

mod args;

mod binding;

mod state;
use crate::state::SharedState;

mod discord;
mod twitch;

mod render;
use binding::bind;
pub use render::Render;

pub mod message;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env"]);
    alto_logger::init_term_logger()?;

    let state = SharedState::default();

    let callables = [bind(("!hello", "says hello"), |msg| async move {
        format!("hello, {}", msg.sender_name())
    })?];

    let twitch = tokio::task::spawn({
        let state = state.clone();
        twitch::create_bot(state, callables.clone())
    });

    let discord = tokio::task::spawn({
        let state = state.clone();
        discord::create_bot(state, callables)
    });

    // TODO not this
    let _ = tokio::join!(twitch, discord);

    Ok(())
}
