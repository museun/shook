use shook_config::Ephemeral;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub address: String,
    pub name: String,
    pub password: Ephemeral,
    pub channel: String,
}
