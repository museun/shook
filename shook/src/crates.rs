use shook_core::prelude::*;

// TODO redo this
enum Match {
    Exact(Crate),
    Closest(Crate),
}

impl Match {
    const fn crate_(&self) -> &Crate {
        match self {
            Match::Exact(crate_) | Match::Closest(crate_) => crate_,
        }
    }
}

impl Render for Match {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        let crate_ = self.crate_();
        let exact = matches!(self, Match::Closest { .. })
            .then_some("# (this was the closest match I could find)")
            .unwrap_or_default();

        match flavor {
            RenderFlavor::Twitch => crate_.format_for_twitch(exact).render(flavor),
            RenderFlavor::Discord => crate_.format_for_discord(exact).render(flavor),
            _ => todo!(),
        }
    }
}

impl Crate {
    fn unwrap_unknown(op: &Option<String>) -> &str {
        op.as_deref().unwrap_or("unknown").trim()
    }

    fn format_for_twitch(&self, exact: &str) -> impl Render {
        let data = format!("{} = {} {}", self.name, self.max_version, exact);
        [&self.description, &self.repository, &self.documentation]
            .into_iter()
            .filter_map(Option::as_ref)
            .zip(["desc", "repo", "docs"])
            .fold(Response::say(data), |resp, (opt, ty)| {
                resp.say(format!("{ty}: {opt}"))
            })
            .finish()
    }

    fn format_for_discord(&self, exact: &str) -> impl Render {
        indoc::formatdoc!(
            r#"```toml
                    {exact}
                    {name} = {version}```
                    **description**: {desc}
                    **repository**: <{repo}>
                    **documenation**: <{docs}>
                "#,
            name = self.name,
            version = self.max_version,
            desc = Self::unwrap_unknown(&self.description),
            repo = Self::unwrap_unknown(&self.repository),
            docs = Self::unwrap_unknown(&self.documentation)
        )
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
struct Crate {
    name: String,
    max_version: String,
    description: Option<String>,
    documentation: Option<String>,
    repository: Option<String>,
    exact_match: bool,
}

struct CratesClient {
    client: reqwest::Client,
    ep: String,
}

impl CratesClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(shook_core::USER_AGENT)
                .build()
                .expect("valid client"),
            ep: String::from("https://crates.io/api/v1/crates"),
        }
    }

    async fn get(&self, query: &str) -> anyhow::Result<Vec<Crate>> {
        #[derive(serde::Deserialize)]
        struct Resp {
            crates: Vec<Crate>,
        }
        let query = &&[("page", "1"), ("per_page", "1"), ("q", query)];
        let resp = self.client.get(&self.ep).query(query).send().await?;
        Ok(resp.json::<Resp>().await?.crates)
    }
}

pub struct Crates;

impl Crates {
    pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        state.insert(CratesClient::new()).await;
        Ok(Binding::create(state, Self)
            .await
            .bind(Self::crates)
            .into_callable())
    }

    async fn crates(self: Arc<Self>, msg: Message) -> impl Render {
        let arg = &msg.args()["name"];

        let client: &CratesClient = &*msg.state().get().await;
        let crates = client.get(arg).await?;

        anyhow::ensure!(!crates.is_empty(), "I cannot find anything for: {arg}");
        let head = crates.first().cloned();

        Ok(crates
            .into_iter()
            .find(|c| c.exact_match)
            .map_or_else(|| Match::Closest(head.unwrap()), Match::Exact))
    }
}
