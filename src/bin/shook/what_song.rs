use std::sync::Arc;

use anyhow::Context;
use rspotify::{
    model::{CurrentlyPlayingType, PlayableItem, TrackId},
    prelude::{Id, OAuthClient},
    AuthCodeSpotify, Credentials, OAuth,
};
use shook::{helix::HelixClient, prelude::*, queue::Queue, IterExt};
use tokio::sync::Mutex;

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    WhatSong::bind(state).await
}

#[derive(Debug, Clone)]
struct Song {
    id: TrackId,
    name: String,
    artists: String,
}

#[derive(Clone)]
pub struct SpotifyClient {
    client: Arc<AuthCodeSpotify>,
    seen: Arc<Mutex<Option<TrackId>>>,
}

impl SpotifyClient {
    async fn new(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
        let credentials = Credentials::new(client_id, client_secret);

        let oauth = OAuth::from_env(rspotify::scopes!(
            "user-read-playback-state",
            "user-read-currently-playing"
        ))
        .with_context(|| "cannot get rspotify oauth pref")?;

        let config = rspotify::Config {
            token_cached: true,
            token_refreshing: true,
            ..rspotify::Config::default()
        };

        let mut auth = AuthCodeSpotify::with_config(credentials, oauth, config);
        let url = auth.get_authorize_url(false)?;
        auth.prompt_for_token(&url).await?; // TODO don't do this

        Ok(Self {
            client: Arc::new(auth),
            seen: <_>::default(),
        })
    }

    async fn try_get_song(&self) -> Option<Song> {
        let song = self
            .client
            .current_playing(None, <Option<Option<_>>>::None)
            .await
            .ok()
            .flatten()?;

        if !song.is_playing || !matches!(song.currently_playing_type, CurrentlyPlayingType::Track) {
            return None;
        }

        let track = match song.item? {
            PlayableItem::Track(track) => track,
            _e => return None,
        };

        let id = track.id?;
        {
            let seen = &mut *self.seen.lock().await;
            if seen.as_ref() == Some(&id) {
                return None;
            }
            seen.replace(id.clone());
        }

        let artists = track.artists.iter().map(|a| &*a.name);

        Some(Song {
            id,
            name: track.name,
            artists: artists.join_with(','),
        })
    }
}

struct Spotify {
    spotify: SpotifyClient,
    queue: Arc<Mutex<Queue<Song>>>,
}

impl Spotify {
    const HISTORY_LIMIT: usize = 5;

    async fn create(state: GlobalState) -> anyhow::Result<Self> {
        let spotify = {
            let config = state.get::<shook::config::Spotify>().await;
            SpotifyClient::new(&config.client_id, &config.client_secret).await?
        };
        let queue = Arc::new(Mutex::new(Queue::with_capacity(Self::HISTORY_LIMIT)));

        tokio::spawn({
            let queue = queue.clone();
            let twitch = state.get_owned::<HelixClient>().await;
            let spotify = spotify.clone();
            let streamer = state.get_owned::<Streamer>().await;
            Self::update_loop(queue, twitch, spotify, streamer)
        });

        Ok(Self { spotify, queue })
    }

    async fn current(&self) -> impl Render {
        if let Some(song) = self.spotify.try_get_song().await {
            let out = Self::format_song(&song);
            self.queue.lock().await.push(song);
            return Ok(out);
        }
        anyhow::bail!("I don't know")
    }

    async fn previous(&self) -> impl Render {
        let queue = self.queue.lock().await;
        let song = queue.last_nth(1).with_context(|| "I don't know")?;
        Ok(Self::format_song(song))
    }

    async fn update_loop(
        queue: Arc<Mutex<Queue<Song>>>,
        twitch: shook::helix::HelixClient,
        spotify: SpotifyClient,
        streamer: Streamer,
    ) {
        loop {
            if let Ok([_stream]) = twitch.get_streams([&streamer.0]).await.as_deref() {
                if let Some(song) = spotify.try_get_song().await {
                    queue.lock().await.push(song);
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }

    fn format_song(song: &Song) -> Simple<impl Render, impl Render> {
        Simple {
            twitch: format!("{} -> {} - {}", song.id.url(), song.artists, song.name),
            discord: song.id.url(),
        }
    }
}

#[derive(serde::Deserialize)]
struct YoutubeItem {
    title: String,
    id: String,
    ts: String,
    duration: String,
}

struct Youtube {
    client: reqwest::Client,
    ep: String,
}

impl Youtube {
    async fn create(state: GlobalState) -> Self {
        Self {
            client: reqwest::Client::new(),
            ep: state.get::<shook::config::Youtube>().await.endpoint.clone(),
        }
    }

    async fn current(&self) -> impl Render {
        let item = self.get("youtube/current").await.ok()?;
        let (ts, dur): (i64, i64) = (item.ts.parse().ok()?, item.duration.parse().ok()?);
        let now = std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .as_secs() as i64;
        if now - ts > dur {
            return None;
        }
        Some(format!("https://youtu.be/{} - {}", item.id, item.title))
    }

    async fn previous(&self) -> impl Render {
        let item = self.get("youtube/previous").await.ok()?;
        Some(format!("https://youtu.be/{} - {}", item.id, item.title))
    }

    async fn get(&self, ep: &str) -> anyhow::Result<YoutubeItem> {
        Ok(self
            .client
            .get(format!("{}/{ep}", self.ep))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

struct WhatSong {
    mode: Arc<Mutex<Mode>>,
    spotify: Spotify,
    youtube: Youtube,
}

#[derive(Default, Copy, Clone)]
enum Mode {
    Spotify,
    Youtube,
    #[default]
    None,
}

impl WhatSong {
    async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let this = Self {
            mode: <Arc<Mutex<_>>>::default(),
            youtube: Youtube::create(state.clone()).await,
            spotify: Spotify::create(state.clone()).await?,
        };

        let reg = state.get().await;
        Ok(Binding::create(&reg, this)
            .bind(Self::current)
            .bind(Self::previous)
            .bind(Self::swap)
            .into_callable())
    }

    async fn current(self: Arc<Self>, _: Message) -> impl Render {
        let mode = self.mode.lock().await;
        match &*mode {
            Mode::Spotify => self.spotify.current().await.boxed(),
            Mode::Youtube => self.youtube.current().await.boxed(),
            Mode::None => "I don't know".boxed(),
        }
    }

    async fn previous(self: Arc<Self>, _: Message) -> impl Render {
        let mode = self.mode.lock().await;
        match &*mode {
            Mode::Spotify => self.spotify.previous().await.boxed(),
            Mode::Youtube => self.youtube.previous().await.boxed(),
            Mode::None => "I don't know".boxed(),
        }
    }

    async fn swap(self: Arc<Self>, msg: Message) -> impl Render {
        if let Some(nope) = msg.require_broadcaster() {
            return Some(nope);
        }

        let mode = match &msg.args()["service"] {
            "spotify" => Mode::Spotify,
            "youtube" => Mode::Youtube,
            "none" | "off" => Mode::None,
            _ => return None,
        };

        let f = |mode| match mode {
            Mode::Spotify => "spotify",
            Mode::Youtube => "youtube",
            Mode::None => "none",
        };

        let old = std::mem::replace(&mut *self.mode.lock().await, mode);
        Some(format!("switched from '{}' to '{}'", f(old), f(mode)).boxed())
    }
}
