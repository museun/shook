use std::sync::Arc;

use shook::config::Config;
use shook_core::prelude::{GlobalState, State, StreamerName};
use shook_helix::{EmoteMap, HelixClient, OAuth};

use persist::{tokio::PersistExt as _, yaml::Yaml};

async fn load_configurations(state: &mut State) -> anyhow::Result<()> {
    let config = Config::load_from_file::<Yaml>("config").await?;

    state.insert(config.twitch);
    state.insert(config.discord);
    state.insert(config.helix);
    state.insert(config.spotify);
    state.insert(config.another_viewer);
    state.insert(config.youtube);
    state.insert(config.user_defined);
    state.insert(config.registry);

    Ok(())
}

async fn load_registry(state: &mut State) -> anyhow::Result<()> {
    let path = state.get_config_path::<shook::config::Registry>();
    let registry = shook_core::help::Registry::load_from_file::<Yaml>(&path)
        .await
        .map(Arc::new)?;
    state.insert(registry);
    Ok(())
}

async fn init_twitch(state: &mut State) -> anyhow::Result<()> {
    let twitch = state.get::<shook_helix::config::Config>()?;
    let streamer = state
        .extract(|config: &shook_twitch::config::Config| config.channel.clone())
        .map(StreamerName)?;

    log::debug!("getting twitch oauth tokens");
    let twitch_oauth = OAuth::create(&twitch.client_id, &twitch.client_secret).await?;

    let twitch_client = HelixClient::new(
        twitch_oauth.get_client_id(), //
        twitch_oauth.get_bearer_token(),
    );

    log::debug!("getting the twitch global emotes");
    let (_, global) = twitch_client.get_global_emotes().await?;
    log::trace!("building emote map");
    let iter = global.iter().map(|c| (&*c.id, &*c.name));
    let emote_map = EmoteMap::default().with_emotes(iter);

    state.insert(twitch_oauth);
    state.insert(twitch_client);
    state.insert(emote_map);
    state.insert(streamer);

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env", ".log.env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let mut state = State::default();

    log::info!("loading configuration");
    load_configurations(&mut state).await?;

    log::info!("loading help");
    load_registry(&mut state).await?;

    log::info!("getting twitch clients");
    init_twitch(&mut state).await?;

    let state = GlobalState::new(state);
    log::trace!("binding callables");
    let callables = [
        shook::Builtin::bind(state.clone()).await?,
        shook::Crates::bind(state.clone()).await?,
        shook::UserDefined::bind(state.clone()).await?,
        shook::WhatSong::bind(state.clone()).await?,
        shook::AnotherViewer::bind(state.clone()).await?,
    ];

    // TODO these should reconnect automatically
    // TODO these should be configurable
    log::debug!("starting local bot");
    let local = tokio::task::spawn({
        let state = state.clone();
        shook_local::create_bot(state, callables.clone())
    });

    log::debug!("starting twitch bot");
    let twitch = tokio::task::spawn({
        let state = state.clone();
        shook_twitch::create_bot(state, callables.clone())
    });

    log::debug!("starting discord bot");
    let discord = tokio::task::spawn({
        let state = state.clone();
        shook_twilight::create_bot(state, callables)
    });

    log::debug!("waiting for both bots to finish");
    // TODO not this
    let _ = tokio::join!(twitch, discord, local);

    Ok(())
}
