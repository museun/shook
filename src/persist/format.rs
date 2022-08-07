use super::AnyhowFut;
use ::serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait PersistFormat {
    fn serialize<'a, T, W>(data: &'a T, out: &'a mut W) -> AnyhowFut<'a, ()>
    where
        T: Serialize + Sync,
        W: AsyncWrite + Unpin + Send + ?Sized;

    fn deserialize<'a, T, R>(input: &'a mut R) -> AnyhowFut<'a, T>
    where
        T: Send + for<'de> Deserialize<'de> + Sync,
        R: AsyncRead + Unpin + Send + ?Sized;

    fn ext() -> &'static str;
    fn with_ext(path: &Path) -> PathBuf {
        path.with_extension(Self::ext())
    }
}
