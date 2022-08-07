use ::serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

type AnyhowFut<'a, T> = BoxFuture<'a, anyhow::Result<T>>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

mod format;
pub use format::PersistFormat;

mod json;
pub use json::Json;

mod yaml;
pub use yaml::Yaml;

mod persist_ext;
pub use persist_ext::PersistExt;

pub trait Persist
where
    for<'de> Self: Serialize + Deserialize<'de>,
{
    fn save<'a, K: PersistFormat>(
        &'a self,
        out: &'a mut (dyn AsyncWrite + Unpin + Send),
    ) -> AnyhowFut<'a, ()>
    where
        Self: Send + Sync,
    {
        Box::pin(K::serialize(self, out))
    }

    fn load<'a, K: PersistFormat>(
        input: &'a mut (dyn AsyncRead + Unpin + Send),
    ) -> BoxFuture<'a, anyhow::Result<Self>>
    where
        Self: Sized + Send + Sync + 'a,
    {
        Box::pin(K::deserialize(input))
    }
}

impl<T: for<'de> Deserialize<'de> + Serialize> Persist for T {}
