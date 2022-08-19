use shook_config::{Ephemeral, Secret};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub client_id: Secret,
    pub client_secret: Ephemeral,
}
