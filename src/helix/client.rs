use super::data;

#[derive(Clone)]
pub struct HelixClient {
    agent: reqwest::Client,
    client_id: String,
    bearer_token: String,
    base: Option<String>,
}

impl HelixClient {
    pub fn new(client_id: &str, bearer_token: &str) -> Self {
        Self::new_with_ep(Option::<String>::None, client_id, bearer_token)
    }

    pub fn new_with_ep(ep: impl Into<Option<String>>, client_id: &str, bearer_token: &str) -> Self {
        let agent = reqwest::Client::builder()
            .user_agent(crate::USER_AGENT)
            // TODO use default headers
            .build()
            .expect("valid client");

        Self {
            agent,
            client_id: client_id.to_string(),
            bearer_token: bearer_token.to_string(),
            base: ep.into().map(Into::into),
        }
    }

    pub async fn get_streams<const N: usize>(
        &self,
        names: [&str; N],
    ) -> anyhow::Result<Vec<data::Stream>> {
        self.get_response(
            "streams",
            &std::iter::repeat("user_login")
                .zip(names)
                .collect::<Vec<_>>(),
        )
        .await
        .map(|data| data.data)
    }

    pub async fn get_global_emotes(&self) -> anyhow::Result<(String, Vec<data::Emote>)> {
        self.get_response("chat/emotes/global", &[])
            .await
            .map(|data| (data.template, data.data))
    }

    pub async fn get_emotes_for(
        &self,
        broadcaster_id: &str,
    ) -> anyhow::Result<(String, Vec<data::Emote>)> {
        self.get_response("chat/emotes/global", &[("broadcaster_id", broadcaster_id)])
            .await
            .map(|data| (data.template, data.data))
    }

    async fn get_response<'k, 'v, T>(
        &self,
        ep: &str,
        query: &[(&'k str, &'v str)],
    ) -> anyhow::Result<data::Data<T>>
    where
        for<'de> T: ::serde::Deserialize<'de> + Send + 'static,
    {
        const BASE_URL: &str = "https://api.twitch.tv/helix";

        let url = format!("{}/{}", self.base.as_deref().unwrap_or(BASE_URL), ep);
        let headers = [
            ("client-id", &*self.client_id),
            ("authorization", &*self.bearer_token),
        ];

        let request = self.agent.get(&url).query(query);
        let request = headers
            .into_iter()
            .fold(request, |req, (k, v)| req.header(k, v));
        let response = request.send().await?;
        Ok(response.json().await?)
    }
}
