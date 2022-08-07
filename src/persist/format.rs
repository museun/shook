use super::AnyhowFut;
use ::serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait PersistFormat {
    fn serialize<'a, T, W>(data: &'a T, out: W) -> AnyhowFut<'a, ()>
    where
        T: Serialize + Sync,
        W: AsyncWrite + Unpin + Send + Sized + 'a;

    fn deserialize<'a, T, R>(input: R) -> AnyhowFut<'a, T>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
        R: AsyncRead + Unpin + Send + Sized + 'a;

    fn ext() -> &'static str;

    fn with_ext(path: &Path) -> PathBuf {
        path.with_extension(Self::ext())
    }
}
