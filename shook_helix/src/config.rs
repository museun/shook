use shook_core::config::{load_from_env, LoadFromEnv, Secret};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Twitch {
    pub client_id: String,
    pub client_secret: Secret<String>,
}

impl LoadFromEnv for Twitch {
    fn load_from_env() -> anyhow::Result<Self> {
        load_from_env(&[
            ("SHAKEN_TWITCH_CLIENT_ID", |t, v| t.client_id = v),
            ("SHAKEN_TWITCH_CLIENT_SECRET", |t, v| {
                t.client_secret = Secret(v)
            }),
        ])
    }
}
