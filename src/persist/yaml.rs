use super::{AnyhowFut, PersistFormat};
use ::serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};

pub struct Yaml;
impl PersistFormat for Yaml {
    fn serialize<'a, T, W>(data: &'a T, mut out: W) -> AnyhowFut<'a, ()>
    where
        T: Serialize + Sync,
        W: AsyncWrite + Unpin + Send + Sized + 'a,
    {
        Box::pin(async move {
            let mut vec = vec![];
            serde_yaml::to_writer(&mut vec, data)?;
            Ok(out.write_all(&vec).await?)
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
            Ok(serde_yaml::from_str(&out)?)
        })
    }

    fn ext() -> &'static str {
        "yml"
    }
}
