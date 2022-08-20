use std::{path::PathBuf, sync::Arc};

use tokio::{io::AsyncWriteExt, sync::Mutex};

pub trait ToRow {
    fn to_row(&self) -> String;
}

pub struct History<T> {
    list: Arc<Mutex<Vec<T>>>,
    path: Arc<PathBuf>,
}

impl<T> Clone for History<T> {
    fn clone(&self) -> Self {
        Self {
            list: Arc::clone(&self.list),
            path: Arc::clone(&self.path),
        }
    }
}

impl<T> History<T> {
    pub async fn load(path: impl Into<PathBuf> + Send) -> anyhow::Result<Self>
    where
        T: std::str::FromStr,
        T::Err: Into<anyhow::Error>,
    {
        let path = path.into();
        let list = match tokio::fs::read_to_string(&path).await {
            Ok(s) => Arc::new(Mutex::new({
                let list: anyhow::Result<Vec<_>> = s
                    .lines()
                    .map(<str>::parse)
                    .map(|s| s.map_err(Into::into))
                    .collect();

                let list = list?;

                log::info!(
                    "loaded {} items from cache at {}",
                    list.len(),
                    path.display()
                );
                list
            })),
            Err(_) => <_>::default(),
        };

        Ok(Self {
            list,
            path: Arc::new(path),
        })
    }

    pub async fn add(&self, item: T) -> anyhow::Result<()>
    where
        T: ToRow + Send + Sync,
    {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&*self.path)
            .await?;

        file.write_all(item.to_row().as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;

        self.list.lock().await.push(item);
        Ok(())
    }

    pub async fn current(&self) -> Option<T>
    where
        T: Send + Clone,
    {
        self.list.lock().await.last().cloned()
    }

    pub async fn previous(&self) -> Option<T>
    where
        T: Send + Clone,
    {
        self.list.lock().await.iter().rev().nth(1).cloned()
    }

    pub async fn all(&self) -> Vec<T>
    where
        T: Send + Clone,
    {
        self.list.lock().await.clone()
    }
}
