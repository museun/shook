use shook_core::config::{load_from_env, LoadFromEnv, Secret};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Spotify {
    pub client_id: String,
    pub client_secret: Secret<String>,
}

impl LoadFromEnv for Spotify {
    fn load_from_env() -> anyhow::Result<Self> {
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

impl LoadFromEnv for AnotherViewer {
    fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_BRAIN_REMOTE_URL", |t, v| t.remote = v),
            ("SHAKEN_BRAIN_GENERATE_TOKEN", |t, v| {
                t.bearer_token = Secret(v)
            }),
        ])
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Youtube {
    pub endpoint: String,
}

impl LoadFromEnv for Youtube {
    fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[("YOUTUBE_HISTORY_SERVER", |t, v| t.endpoint = v)])
    }
}
