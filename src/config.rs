use anyhow::Context;

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Secret<T>(pub T);

impl From<String> for Secret<String> {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Debug for Secret<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{{len = {}}}"#, self.0.len())
    }
}

impl std::fmt::Display for Secret<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> std::ops::Deref for Secret<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type Assign<T> = fn(&mut T, String);
fn load_from_env<T: Default + std::fmt::Debug>(keys: &[(&str, Assign<T>)]) -> anyhow::Result<T> {
    let get = |key| {
        log::trace!("looking up {key}");
        let res = std::env::var(key);
        res.with_context(|| anyhow::anyhow!("key '{key}' was not found"))
    };

    log::trace!("loading env vars for: {}", std::any::type_name::<T>());

    let this = keys.iter().try_fold(T::default(), |mut this, (key, func)| {
        func(&mut this, get(key)?);
        Ok(this)
    });

    if let Ok(this) = &this {
        log::debug!("created: {:?}", this);
    }
    this
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Irc {
    pub addr: String,
    pub name: String,
    pub pass: Secret<String>,
    pub channel: String,
}

impl Irc {
    pub fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_TWITCH_IRC_ADDRESS", |t, v| t.addr = v),
            ("SHAKEN_TWITCH_NAME", |t, v| t.name = v),
            ("SHAKEN_TWITCH_OAUTH_TOKEN", |t, v| t.pass = Secret(v)),
            ("SHAKEN_TWITCH_CHANNEL", |t, v| t.channel = v),
        ])
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Twitch {
    pub client_id: String,
    pub client_secret: Secret<String>,
}

impl Twitch {
    pub fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_TWITCH_CLIENT_ID", |t, v| t.client_id = v),
            ("SHAKEN_TWITCH_CLIENT_SECRET", |t, v| {
                t.client_secret = Secret(v)
            }),
        ])
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Discord {
    pub oauth_token: Secret<String>,
}

impl Discord {
    pub fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[("SHAKEN_DISCORD_OAUTH_TOKEN", |t, v| {
            t.oauth_token = Secret(v)
        })])
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Spotify {
    pub client_id: String,
    pub client_secret: Secret<String>,
}

impl Spotify {
    pub fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_SPOTIFY_CLIENT_ID", |t, v| t.client_id = v),
            ("SHAKEN_SPOTIFY_CLIENT_SECRET", |t, v| {
                t.client_secret = Secret(v)
            }),
        ])
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AnotherViewer {
    pub remote: String,
    pub bearer_token: Secret<String>,
}

impl AnotherViewer {
    pub fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_BRAIN_REMOTE_URL", |t, v| t.remote = v),
            ("SHAKEN_BRAIN_GENERATE_TOKEN", |t, v| {
                t.bearer_token = Secret(v)
            }),
        ])
    }
}
