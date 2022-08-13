use shook_config::{load_from_env, LoadFromEnv, Secret};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Irc {
    pub addr: String,
    pub name: String,
    pub pass: Secret<String>,
    pub channel: String,
}

impl LoadFromEnv for Irc {
    fn load_from_env() -> anyhow::Result<Self> {
        load_from_env([
            ("SHAKEN_TWITCH_IRC_ADDRESS", |t, v| t.addr = v),
            ("SHAKEN_TWITCH_NAME", |t, v| t.name = v),
            ("SHAKEN_TWITCH_OAUTH_TOKEN", |t, v| t.pass = Secret(v)),
            ("SHAKEN_TWITCH_CHANNEL", |t, v| t.channel = v),
        ])
    }
}
