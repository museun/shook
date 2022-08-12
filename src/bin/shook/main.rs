use shook::{
    discord,
    help::Registry,
    prelude::{GlobalState, State, Streamer},
    twitch,
};

use persist::{tokio::PersistExt, yaml::Yaml};

#[allow(dead_code)]
mod another_viewer;
mod builtin;
mod crates;
mod local;
mod user_defined;
mod what_song;

fn load_config(state: &mut State) -> anyhow::Result<()> {
    use shook::config::*;
    fn load<F: LoadFromEnv + Send + Sync + 'static>(state: &mut State) -> anyhow::Result<()> {
        F::load_from_env().map(|config| state.insert(config))
    }
    macro_rules! load {
        ($($ty:ty)*) => {
            $(load::<$ty>(state)?;)*
        };
    }

    load! {
        Irc
        Twitch
        Spotify
        Discord
        AnotherViewer
        Youtube
    }

    log::info!("succesfully loaded env");
    Ok(())
}

async fn load_help(state: &mut State) -> anyhow::Result<()> {
    let registry = Registry::load_from_file::<Yaml>("default_help").await?;
    state.insert(registry);
    Ok(())
}

async fn init_twitch(state: &mut State) -> anyhow::Result<()> {
    use shook::{config, helix::*};
    let twitch = state.get::<config::Twitch>()?;

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

    // TODO change this
    state.insert(Streamer("museun".into()));

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env"]);
    alto_logger::TermLogger::new(
        alto_logger::Options::default()
            .with_time(alto_logger::TimeConfig::relative_now())
            .with_style(alto_logger::StyleConfig::SingleLine),
    )?
    .init()?;

    let mut state = State::default();
    log::info!("loading configuration");
    load_config(&mut state)?;

    log::info!("loading help");
    load_help(&mut state).await?;

    log::info!("getting twitch clients");
    init_twitch(&mut state).await?;

    let state = GlobalState::new(state);
    log::trace!("binding callables");
    let callables = [
        // another_viewer::bind(state.clone()).await?,
        builtin::bind(state.clone()).await?,
        crates::bind(state.clone()).await?,
        user_defined::bind(state.clone()).await?,
        what_song::bind(state.clone()).await?,
    ];

    log::debug!("starting local bot");
    let local = tokio::task::spawn({
        let state = state.clone();
        local::create_bot(state, callables.clone())
    });

    log::debug!("starting twitch bot");
    let twitch = tokio::task::spawn({
        let state = state.clone();
        // TODO this should retry and reconnect
        twitch::create_bot(state, callables.clone())
    });

    log::debug!("starting discord bot");
    let discord = tokio::task::spawn({
        let state = state.clone();
        // TODO what are the error states for this?
        discord::create_bot(state, callables)
    });

    log::debug!("waiting for both bots to finish");
    // TODO not this
    let _ = tokio::join!(twitch, discord, local);

    Ok(())
}
