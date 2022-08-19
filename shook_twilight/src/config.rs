use shook_config::Ephemeral;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub oauth_token: Ephemeral,
}
