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

mod toml;
pub use self::toml::Toml;

mod ron {
    use super::{AnyhowFut, PersistFormat};
    use ::serde::{Deserialize, Serialize};
    use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};

    pub struct Ron;

    impl PersistFormat for Ron {
        fn serialize<'a, T, W>(data: &'a T, mut out: W) -> AnyhowFut<'a, ()>
        where
            T: Serialize + Sync,
            W: AsyncWrite + Unpin + Send + Sized + 'a,
        {
            Box::pin(async move {
                let data = ron::ser::to_string_pretty(data, {
                    ron::ser::PrettyConfig::default()
                        .separate_tuple_members(false)
                        .indentor(String::from("  "))
                })?;
                Ok(out.write_all(data.as_bytes()).await?)
            })
        }

        fn deserialize<'a, T, R>(mut input: R) -> AnyhowFut<'a, T>
        where
            T: for<'de> Deserialize<'de> + Send + Sync,
            R: AsyncRead + Unpin + Send + Sized + 'a,
        {
            Box::pin(async move {
                let mut out = String::new();
                input.read_to_string(&mut out).await?;
                Ok(ron::from_str(&out)?)
            })
        }

        fn ext() -> &'static str {
            "ron"
        }
    }
}

pub use self::ron::Ron;

mod lexpr;
pub use self::lexpr::Lexpr;

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
