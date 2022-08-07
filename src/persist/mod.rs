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

mod ext;
pub use ext::PersistExt;

type Write = dyn AsyncWrite + Unpin + Send;
type Read = dyn AsyncRead + Unpin + Send;

pub trait Persist
where
    for<'de> Self: Serialize + Deserialize<'de>,
    Self: Send + Sync,
{
    fn save<'a, K>(&'a self, out: &'a mut Write) -> AnyhowFut<'a, ()>
    where
        K: PersistFormat,
    {
        Box::pin(K::serialize(self, out))
    }

    fn load<'a, K>(input: &'a mut Read) -> AnyhowFut<'a, Self>
    where
        Self: Sized + 'a,
        K: PersistFormat,
    {
        Box::pin(K::deserialize(input))
    }
}

impl<T> Persist for T where T: for<'de> Deserialize<'de> + Serialize + Send + Sync {}
