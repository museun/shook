use std::path::{Path, PathBuf};

use shook_config::{Ephemeral, Secret};
use shook_core::ConfigPath;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Spotify {
    pub client_id: Secret,
    pub client_secret: Ephemeral,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AnotherViewer {
    pub endpoint: Secret,
    pub bearer_token: Ephemeral,
    pub filter_patterns_path: PathBuf,
}

impl ConfigPath for AnotherViewer {
    fn file_path(&self) -> &Path {
        &self.filter_patterns_path
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Youtube {
    pub endpoint: Secret,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct UserDefined {
    pub user_defined_path: PathBuf,
}

impl ConfigPath for UserDefined {
    fn file_path(&self) -> &Path {
        &self.user_defined_path
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Registry {
    pub registry_path: PathBuf,
}

impl ConfigPath for Registry {
    fn file_path(&self) -> &Path {
        &self.registry_path
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub twitch: shook_twitch::config::Config,
    pub discord: shook_twilight::config::Config,
    pub helix: shook_helix::config::Config,

    pub spotify: self::Spotify,
    pub another_viewer: self::AnotherViewer,
    pub youtube: self::Youtube,
    pub user_defined: self::UserDefined,
    pub registry: self::Registry,
}

impl Config {
    pub fn default_config() -> Self {
        Self {
            twitch: shook_twitch::config::Config {
                address: String::from("irc.chat.twitch.tv:6667"),
                name: String::from("shaken_bot"),
                password: Ephemeral::key("SHAKEN_TWITCH_OAUTH_TOKEN"),
                channel: String::from("#museun"),
            },
            discord: shook_twilight::config::Config {
                oauth_token: Ephemeral::key("SHAKEN_DISCORD_OAUTH_TOKEN"),
            },
            helix: shook_helix::config::Config {
                client_id: Secret::key("SHAKEN_TWITCH_CLIENT_ID"),
                client_secret: Ephemeral::key("SHAKEN_TWITCH_CLIENT_SECRET"),
            },
            spotify: Spotify {
                client_id: Secret::key("SHAKEN_SPOTIFY_CLIENT_ID"),
                client_secret: Ephemeral::key("SHAKEN_SPOTIFY_CLIENT_SECRET"),
            },
            another_viewer: AnotherViewer {
                endpoint: Secret::key("SHAKEN_BRAIN_REMOTE"),
                bearer_token: Ephemeral::key("SHAKEN_BRAIN_GENERATE_TOKEN"),
                filter_patterns_path: PathBuf::from("./data/filter_patterns.yaml"),
            },
            youtube: Youtube {
                endpoint: Secret::key("SHAKEN_WHAT_SONG_REMOTE"),
            },
            user_defined: UserDefined {
                user_defined_path: PathBuf::from("./data/user_defined.json"),
            },
            registry: Registry {
                registry_path: PathBuf::from("./data/registry.yaml"),
            },
        }
    }
}
