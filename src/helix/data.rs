#[derive(::serde::Deserialize)]
pub struct Data<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub template: String,
}

#[derive(Clone, Debug, ::serde::Deserialize)]
pub struct Stream {
    #[serde(deserialize_with = "crate::serde::from_str")]
    pub id: u64,

    #[serde(deserialize_with = "crate::serde::from_str")]
    pub user_id: u64,
    pub user_name: String,

    #[serde(deserialize_with = "crate::serde::from_str")]
    pub game_id: u64,
    pub title: String,
    pub viewer_count: u64,

    #[serde(deserialize_with = "crate::serde::assume_utc_date_time")]
    pub started_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, ::serde::Deserialize)]
pub struct Emote {
    pub id: String,
    pub name: String,
}
