use std::time::Duration;

use fastrand_ext::IterExt as _;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::ser::SerializeSeq;
use shook_config::{Ephemeral, Secret};
use shook_core::{prelude::*, IterExt as _, PersistFromConfig};
use shook_helix::EmoteMap;
use tokio::{sync::Mutex, time::Instant};

#[derive(Clone, Debug, Default)]
struct Patterns {
    list: Vec<Regex>,
}

impl<'de> serde::Deserialize<'de> for Patterns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let list = <Vec<std::borrow::Cow<'_, str>>>::deserialize(deserializer)?;
        list.into_iter()
            .map(|re| Regex::new(&*re).map_err(D::Error::custom))
            .collect::<Result<_, D::Error>>()
            .map(|list| Self { list })
    }
}

impl serde::Serialize for Patterns {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.list.len()))?;
        self.list
            .iter()
            .map(|c| c.as_str())
            .try_for_each(|re| seq.serialize_element(re))?;
        seq.end()
    }
}

impl PersistFromConfig for Patterns {
    type ConfigPath = crate::config::AnotherViewer;
}

pub struct AnotherViewer {
    last: Mutex<Instant>,
    emote_map: EmoteMap,
    client: reqwest::Client,
    bearer_token: Ephemeral,
    endpoint: Secret,
    patterns: Patterns,
}

impl AnotherViewer {
    pub async fn bind(state: GlobalState) -> anyhow::Result<SharedCallable> {
        let crate::config::AnotherViewer {
            endpoint,
            bearer_token,
            ..
        } = state.get_owned().await;

        let patterns = Patterns::load_from_file(&state).await?;
        let emote_map = state.get_owned().await;

        let this = Self {
            last: Mutex::new(Instant::now()),
            client: reqwest::Client::new(),
            emote_map,
            bearer_token,
            endpoint,
            patterns,
        };

        Ok(Binding::create(state, this)
            .await
            .bind(Self::speak)
            .listen(Self::listen)
            .into_callable())
    }

    async fn speak(self: Arc<Self>, msg: Message) -> impl Render {
        let ctx = msg.args().get("context");
        self.generate(ctx).await.map(Response::reply)
    }

    async fn listen(self: Arc<Self>, msg: Message) -> impl Render {
        if msg.data().starts_with('!') {
            return None;
        }

        let _ = self.train(msg.data()).await;
        let s = msg.data().split_ascii_whitespace().collect::<Vec<_>>();

        if let Some(msg) = self.try_mention(msg.sender_name(), &s).await {
            return Some(msg.boxed());
        }

        if let Some(msg) = self.try_kappa(&s).await {
            return Some(msg.boxed());
        }

        if !self.check_timeout().await {
            return None;
        }
        self.generate(None).await.map(|r| r.boxed())
    }

    async fn try_mention<'a>(&self, sender: &str, ctx: &'a [&'a str]) -> Option<impl Render> {
        if Self::contains_bot_name(ctx) {
            return self.generate(Some(sender)).await.map(Response::reply);
        }
        None
    }

    async fn try_kappa<'a>(&self, ctx: &'a [&'a str]) -> Option<impl Render> {
        let kappa = ctx
            .iter()
            .filter(|c| self.emote_map.has(c))
            .choose(&fastrand::Rng::new())?;
        self.generate(Some(kappa)).await
    }

    async fn check_timeout(&self) -> bool {
        const COOLDOWN: Duration = Duration::from_secs(60);
        let mut last = self.last.lock().await;
        if last.elapsed() >= COOLDOWN {
            *last = Instant::now();
            return true;
        }
        false
    }

    async fn train(&self, data: &str) -> Option<()> {
        #[derive(serde::Serialize)]
        struct Req {
            data: String,
        }

        let data = self.process_data(data);
        self.client
            .post(format!("{}/shaken/brain/train", &*self.endpoint))
            .bearer_auth(&*self.bearer_token)
            .json(&Req { data })
            .send()
            .await
            .ok()
            .map(drop)
    }

    async fn generate(&self, context: Option<&str>) -> Option<String> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: String,
        }

        #[derive(Debug, serde::Serialize)]
        struct Req {
            min: usize,
            max: usize,
            context: Option<String>,
        }

        let req = Req {
            min: 3,
            max: 50,
            context: context.map(ToString::to_string),
        };

        self.client
            .get(format!("{}/shaken/brain/generate", &*self.endpoint))
            .json(&req)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .ok()?
            .json::<Resp>()
            .await
            .ok()
            .map(|c| self.process_data(&c.data))
    }

    fn process_data(&self, data: &str) -> String {
        let data = &mut String::from(data);
        [
            Self::filter_mentions,
            Self::filter_annoying_patterns,
            Self::filter_urls,
            // this has to be last
            Self::collapse_whitespace,
        ]
        .into_iter()
        .for_each(|f| *data = f(self, &*data));
        data.to_string()
    }

    fn filter_annoying_patterns(&self, data: &str) -> String {
        self.patterns
            .list
            .iter()
            .fold(String::with_capacity(data.len()), |mut text, re| {
                text.push_str(&*re.replace_all(data, ""));
                text
            })
    }

    fn filter_mentions(&self, input: &str) -> String {
        static PATTERN: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#"((!|@).+?\b)|(\bshaken(_bot)?\b)"#).unwrap());
        PATTERN.replace_all(input, "").to_string()
    }

    fn filter_urls(&self, input: &str) -> String {
        input
            .split_ascii_whitespace()
            .filter(|c| url::Url::parse(c).is_err())
            .join_with(' ')
    }

    fn collapse_whitespace(&self, input: &str) -> String {
        static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\s{2,}"#).unwrap());
        PATTERN.replace_all(input, " ").to_string()
    }

    fn contains_bot_name(ctx: &[&str]) -> bool {
        fn match_name(s: &str) -> bool {
            const HEAD: &[char] = &['(', '[', '{', '@', '\'', '"'];
            const TAIL: &[char] = &['"', '\'', ',', '.', '?', ':', ';', '!', '}', ']', ')'];
            matches!(
                s.trim_start_matches(HEAD).trim_end_matches(TAIL),
                "shaken" | "shaken_bot"
            )
        }

        ctx.iter().copied().any(match_name)
    }
}
