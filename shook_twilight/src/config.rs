use shook_config::{load_from_env, LoadFromEnv, Secret};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Discord {
    pub oauth_token: Secret<String>,
}

impl LoadFromEnv for Discord {
    fn load_from_env() -> anyhow::Result<Self> {
        load_from_env([("SHAKEN_DISCORD_OAUTH_TOKEN", |t, v| {
            t.oauth_token = Secret(v)
        })])
    }
}
