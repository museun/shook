use std::path::Path;

use anyhow::Context;

use reqwest::Url;

use super::{Client, Item};
use crate::history::History;

#[derive(Clone)]
pub struct State {
    pub history: History<Item>,
    client: Client,
}

impl State {
    pub async fn new(client: Client, path: impl AsRef<Path> + Send) -> anyhow::Result<Self> {
        let history = History::load(path.as_ref()).await?;
        Ok(Self { history, client })
    }

    pub async fn add(&self, url: &str, ts: i64) -> anyhow::Result<()> {
        let url = Url::parse(url)?;
        let (_, id) = url
            .query_pairs()
            .find(|(c, _)| c == "v")
            .with_context(|| "cannot find id")?;

        log::debug!("adding: {id} @ {ts}");

        let item = self.client.lookup_id(&*id, ts).await?;
        self.history.add(item).await
    }
}
