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

pub async fn dispatch_and_render(
    s: &[SharedCallable],
    msg: &Message,
    flavor: RenderFlavor,
) -> Vec<Response> {
    Dispatch::new(s).into_render(msg).await.render(flavor)
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum RenderFlavor {
    Twitch,
    Discord,
}

pub trait Render
where
    Self: Send + Sync + 'static,
{
    fn render(&self, flavor: RenderFlavor) -> Vec<Response>;

    fn boxed(self) -> Box<dyn Render>
    where
        Self: Sized,
    {
        Box::new(self)
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
        self.to_string().render(flavor)
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
