use std::sync::Arc;

use anyhow::Context;

use crate::history::ToRow;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub duration: String,
    pub ts: String,
}

impl ToRow for Item {
    fn to_row(&self) -> String {
        format!("{},{},{},{}", self.id, self.title, self.duration, self.ts,)
    }
}

impl std::str::FromStr for Item {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let mut iter = input.splitn(4, ',').map(ToString::to_string);

        Ok(Self {
            id: iter.next().with_context(|| "missing field `id`")?,
            title: iter.next().with_context(|| "missing field `title`")?,
            duration: iter.next().with_context(|| "missing field `duration`")?,
            ts: iter.next().with_context(|| "missing field `ts`")?,
        })
    }
}

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    api_key: Arc<str>,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: Arc::from(api_key),
        }
    }

    pub async fn lookup_id(&self, id: &str, ts: i64) -> anyhow::Result<Item> {
        #[derive(serde::Deserialize)]
        struct Response {
            items: Vec<Item>,
        }

        #[derive(serde::Deserialize, Debug)]
        struct Item {
            snippet: Snippet,
            #[serde(rename = "contentDetails")]
            details: ContentDetails,
        }

        #[derive(serde::Deserialize, Debug)]
        struct Snippet {
            title: String,
        }

        #[derive(serde::Deserialize, Debug)]
        struct ContentDetails {
            duration: String,
        }

        const PARTS: (&str, &str) = ("part", "snippet,contentDetails");
        const FIELDS: (&str, &str) = (
            "fields",
            "items(id, snippet(title), contentDetails(duration))",
        );

        let query = &[PARTS, FIELDS, ("key", &*self.api_key), ("id", id)];

        let mut resp: Response = self
            .client
            .get("https://www.googleapis.com/youtube/v3/videos")
            .query(query)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if !resp.items.is_empty() {
            let item = resp.items.swap_remove(0);
            log::debug!("item: {:#?}", item);
            let item = self::Item {
                title: item.snippet.title.to_string(),
                duration: from_iso8601(&item.details.duration).to_string(),
                id: id.to_string(),
                ts: ts.to_string(),
            };
            return Ok(item);
        }

        anyhow::bail!("invalid response")
    }
}

fn from_iso8601(period: &str) -> i64 {
    let parse = |s, e| period[s + 1..e].parse::<i64>().unwrap_or_default();
    period
        .chars()
        .enumerate()
        .fold((0, 0), |(a, p), (i, c)| match c {
            c if c.is_numeric() => (a, p),
            'H' => (a + parse(p, i) * 60 * 60, i),
            'M' => (a + parse(p, i) * 60, i),
            'S' => (a + parse(p, i), i),
            _ => (a, i),
        })
        .0
}
