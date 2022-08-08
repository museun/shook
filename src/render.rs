use crate::{
    callable::{Dispatch, SharedCallable},
    prelude::Message,
};

#[derive(Clone, Debug)]
pub enum Response {
    Say(String),
    Reply(String),
    Problem(String),
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }
    pub fn say(data: impl Into<String>) -> ResponseBuilder {
        Self::builder().say(data)
    }
    pub fn reply(data: impl Into<String>) -> ResponseBuilder {
        Self::builder().reply(data)
    }
    pub fn problem(data: impl Into<String>) -> ResponseBuilder {
        Self::builder().problem(data)
    }
}

#[derive(Default)]
pub struct ResponseBuilder(Vec<Response>);
impl ResponseBuilder {
    pub fn say(mut self, data: impl Into<String>) -> Self {
        self.0.push(Response::Say(data.into()));
        self
    }
    pub fn reply(mut self, data: impl Into<String>) -> Self {
        self.0.push(Response::Reply(data.into()));
        self
    }
    pub fn problem(mut self, data: impl Into<String>) -> Self {
        self.0.push(Response::Problem(data.into()));
        self
    }
    pub fn finish(self) -> Vec<Response> {
        self.0
    }
}

impl Render for ResponseBuilder {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        self.0.render(flavor)
    }
}

pub async fn dispatch_and_render(
    callables: &[SharedCallable],
    msg: &Message,
    flavor: RenderFlavor,
) -> Vec<Response> {
    Dispatch::new(callables)
        .into_render(msg)
        .await
        .render(flavor)
}

macro_rules! md_generate {
    ($($ident:ident: $head:expr => $tail:expr)*) => {
        $(
        pub struct $ident<T>(pub T);
        impl<T: std::fmt::Display> std::fmt::Display for $ident<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{head}{}{tail}", self.0, head = $head, tail = $tail)
            }
        }
        impl<T: std::fmt::Display + Send + Sync> Render for $ident<T> {
            fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
                match flavor {
                    RenderFlavor::Twitch => self.0.to_string().render(flavor),
                    RenderFlavor::Discord => self.to_string().render(flavor),
                }
            }
        }
        )*
    };
}

md_generate! {
    Code:      "`"  => "`"
    Bold:      "**" => "**"
    Underline: "_"  => "__"
    Italics:   "_"  => "_"
    Strikeout: "~"  => "~"
    Hidden:    "<"  => ">"
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum RenderFlavor {
    Twitch,
    Discord,
}

pub trait Render
where
    Self: Send + Sync,
{
    fn render(&self, flavor: RenderFlavor) -> Vec<Response>;
    fn boxed(self) -> Box<dyn Render>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<L: Render, R: Render> Render for (L, R) {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        match flavor {
            RenderFlavor::Twitch => self.0.render(flavor),
            RenderFlavor::Discord => self.1.render(flavor),
        }
    }
}

pub struct Simple<L, R> {
    pub twitch: L,
    pub discord: R,
}

impl<L: Render, R: Render> Render for Simple<L, R> {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        match flavor {
            RenderFlavor::Twitch => self.twitch.render(flavor),
            RenderFlavor::Discord => self.discord.render(flavor),
        }
    }
}

pub type BoxedRender = Box<dyn Render>;

impl Render for BoxedRender {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        (**self).render(flavor)
    }

    #[inline(always)]
    fn boxed(self) -> Self {
        self
    }
}

impl<T: Render> Render for &T {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        (*self).render(flavor)
    }
}

impl Render for Response {
    fn render(&self, _: RenderFlavor) -> Vec<Response> {
        vec![self.clone()]
    }
}

impl<T: Render, const N: usize> Render for [T; N] {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        self.iter().flat_map(|this| this.render(flavor)).collect()
    }
}

impl<T: Render> Render for Vec<T> {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        self.iter().flat_map(|this| this.render(flavor)).collect()
    }
}

impl Render for str {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        if self.trim().is_empty() {
            return vec![];
        }
        self.to_string().render(flavor)
    }
}

impl Render for &'static str {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        if self.trim().is_empty() {
            return vec![];
        }
        (*self).render(flavor)
    }
}

impl Render for String {
    fn render(&self, _: RenderFlavor) -> Vec<Response> {
        if self.trim().is_empty() {
            return vec![];
        }
        vec![Response::Say(self.to_string())]
    }
}

impl Render for () {
    fn render(&self, _: RenderFlavor) -> Vec<Response> {
        vec![]
    }
}

impl<T: Render> Render for anyhow::Result<T> {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        match self {
            Ok(r) => r.render(flavor),
            Err(e) => vec![Response::Problem(e.to_string())],
        }
    }
}

impl<T: Render> Render for Option<T> {
    fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
        self.as_ref()
            .map(|this| this.render(flavor))
            .unwrap_or_default()
    }
}
