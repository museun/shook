use crate::{state::GlobalState, BoxedFuture};
use persist::{json::JsonPretty, tokio::PersistExt, yaml::Yaml};
use std::path::Path;

pub trait ConfigPath {
    fn file_path(&self) -> &Path;
}

pub trait PersistFromConfig:
    serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync
{
    type ConfigPath: ConfigPath + Send + Sync + 'static;

    fn save_to_file<'a>(&'a self, state: &'a GlobalState) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let path = state.get_config_path::<Self::ConfigPath>().await;
            match path.extension() {
                Some(ext) if "json" == ext => {
                    Ok(<_ as PersistExt>::save_to_file::<JsonPretty>(self, path).await?)
                }
                Some(ext) if "yaml" == ext => {
                    Ok(<_ as PersistExt>::save_to_file::<Yaml>(self, path).await?)
                }
                Some(ext) => anyhow::bail!("invalid file extension: {}", ext.to_string_lossy()),
                None => {
                    anyhow::bail!("cannot find config file for: {}", path.to_string_lossy())
                }
            }
        })
    }

    fn load_from_file(state: &GlobalState) -> BoxedFuture<'_, anyhow::Result<Self>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let path = state.get_config_path::<Self::ConfigPath>().await;
            match path.extension() {
                Some(ext) if "json" == ext => {
                    Ok(<Self as PersistExt>::load_from_file::<JsonPretty>(path).await?)
                }
                Some(ext) if "yaml" == ext => {
                    Ok(<Self as PersistExt>::load_from_file::<Yaml>(path).await?)
                }
                Some(ext) => anyhow::bail!("invalid file extension: {}", ext.to_string_lossy()),
                None => {
                    anyhow::bail!("cannot find config file for: {}", path.to_string_lossy())
                }
            }
        })
    }
}
