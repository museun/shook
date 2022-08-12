use crate::history::{History, ToRow};

pub type State = History<Item>;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Item {
    pub id: String,
    pub title: String,
}

impl std::str::FromStr for Item {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> anyhow::Result<Self> {
        match input.split_once(',') {
            Some((id, title)) => Ok(Self {
                id: id.into(),
                title: title.into(),
            }),
            None => anyhow::bail!("invalid entry"),
        }
    }
}

impl ToRow for Item {
    fn to_row(&self) -> String {
        format!("{},{}", self.id, self.title)
    }
}
