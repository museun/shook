use shook::{discord, prelude::GlobalState, twitch};

mod builtin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env"]);
    alto_logger::init_term_logger()?;

    let state = GlobalState::default();

    let callables = [
        builtin::Builtin::bind(state.clone()).await?, //
    ];

    let twitch = tokio::task::spawn({
        let state = state.clone();
        // TODO this should retry and reconnect
        twitch::create_bot(state, callables.clone())
    });

    let discord = tokio::task::spawn({
        let state = state.clone();
        // TODO what are the error states for this?
        discord::create_bot(state, callables)
    });

    // TODO not this
    let _ = tokio::join!(twitch, discord);

    Ok(())
}
