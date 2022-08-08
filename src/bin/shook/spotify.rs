use anyhow::Context;
use rspotify::{
    model::{CurrentlyPlayingType, PlayableItem, TrackId},
    prelude::{Id, OAuthClient},
    AuthCodeSpotify, Credentials, OAuth,
};
use shook::{prelude::*, queue::Queue, IterExt};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Song {
    pub id: TrackId,
    pub name: String,
    pub artists: String,
    pub duration: Duration,
    pub progress: Duration,
}

#[derive(Clone)]
pub struct SpotifyClient {
    client: Arc<AuthCodeSpotify>,
    seen: Arc<Mutex<Option<TrackId>>>,
}

impl SpotifyClient {
    pub async fn new(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
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

    pub async fn try_get_song(&self) -> Option<Song> {
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
            _ => return None,
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
            duration: track.duration,
            progress: song.progress?,
        })
    }
}

pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
    Spotify::bind(state).await
}

struct Spotify {
    spotify: SpotifyClient,
    queue: Arc<Mutex<Queue<Song>>>,
}

impl Spotify {
    const HISTORY_LIMIT: usize = 5;

    async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let spotify = {
            let config: &shook::config::Spotify = &*state.get().await;
            SpotifyClient::new(&config.client_id, &config.client_secret).await?
        };

        let queue = Arc::new(Mutex::new(Queue::with_capacity(Self::HISTORY_LIMIT)));

        let streamer = state.get::<Streamer>().await.clone();
        let _ = tokio::task::spawn({
            let queue = Arc::clone(&queue);
            let twitch = state.get::<shook::helix::HelixClient>().await.clone();
            Self::update_loop(queue, twitch, spotify.clone(), streamer)
        });

        let registry = state.get().await;
        Ok(Binding::create(&registry, Self { spotify, queue })
            .bind("spotify::song", Self::current)
            .bind("spotify::previous", Self::previous)
            // .bind(
            //     cmd("!recent").help("lists recently played songs from spotify"),
            //     Self::recent,
            // )
            .into_callable())
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

    async fn current(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_streaming().await?;

        if let Some(song) = self.queue.lock().await.last() {
            return Ok(Self::format_song(song));
        }

        if let Some(song) = self.spotify.try_get_song().await {
            let out = Self::format_song(&song);
            self.queue.lock().await.push(song);
            return Ok(out);
        }

        anyhow::bail!("I don't know")
    }

    async fn previous(self: Arc<Self>, msg: Message) -> impl Render {
        msg.require_streaming().await?;

        let queue = self.queue.lock().await;
        let song = queue.last_nth(1).with_context(|| "I don't know")?;
        Ok(Self::format_song(song))
    }

    // async fn recent(self: Arc<Self>, msg: Message) -> impl Render {
    //     msg.require_streaming().await?;

    //     let queue = self.queue.lock().await;
    //     anyhow::ensure!(!queue.is_empty(), "I don't know");

    //     queue
    //         .iter()
    //         .enumerate()
    //         .take(Self::HISTORY_LIMIT)
    //         .fold(Response::empty(), |resp, (i, e)| {
    //             let t;
    //             let s: &dyn std::fmt::Display = match i {
    //                 0 => &"current",
    //                 1 => &"previous",
    //                 n => {
    //                     t = format!("previous -{}", n - 1);
    //                     &t
    //                 }
    //             };
    //             resp.say(format!("{}: {}", s, e,))
    //         })
    //         .ok()
    // }
}
