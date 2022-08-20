#[derive(serde::Deserialize, serde::Serialize)]
pub struct Train {
    pub data: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Generate {
    pub min: usize,
    pub max: usize,
    pub query: Option<String>,
}

impl Default for Generate {
    fn default() -> Self {
        Self {
            min: 3,
            max: 50,
            query: None,
        }
    }
}
