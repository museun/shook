pub trait Render
where
    Self: Send + Sync + 'static,
{
    fn render_twitch(&self) -> Vec<Response>;
    fn render_discord(&self) -> Vec<Response> {
        self.render_twitch()
    }

    fn boxed(self) -> Box<dyn Render>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

pub type BoxedRender = Box<dyn Render>;

impl Render for BoxedRender {
    fn render_twitch(&self) -> Vec<Response> {
        (&**self).render_twitch()
    }

    fn render_discord(&self) -> Vec<Response> {
        (&**self).render_discord()
    }
}

#[derive(Clone)]
pub enum Response {
    Say(String),
    Reply(String),
    Problem(String),
}

impl Render for Response {
    fn render_twitch(&self) -> Vec<Response> {
        vec![self.clone()]
    }
}

impl<T: Render, const N: usize> Render for [T; N] {
    fn render_twitch(&self) -> Vec<Response> {
        self.iter().flat_map(<_>::render_twitch).collect()
    }

    fn render_discord(&self) -> Vec<Response> {
        self.iter().flat_map(<_>::render_discord).collect()
    }
}

impl<T: Render> Render for Vec<T> {
    fn render_twitch(&self) -> Vec<Response> {
        self.iter().flat_map(<_>::render_twitch).collect()
    }

    fn render_discord(&self) -> Vec<Response> {
        self.iter().flat_map(<_>::render_discord).collect()
    }
}

impl Render for str {
    fn render_twitch(&self) -> Vec<Response> {
        self.to_string().render_twitch()
    }
}

impl Render for &'static str {
    fn render_twitch(&self) -> Vec<Response> {
        self.to_string().render_twitch()
    }
}

impl Render for String {
    fn render_twitch(&self) -> Vec<Response> {
        vec![Response::Say(self.to_string())]
    }
}

impl Render for bool {
    fn render_twitch(&self) -> Vec<Response> {
        match *self {
            true => "true",
            false => "false",
        }
        .render_twitch()
    }
}

impl Render for () {
    fn render_twitch(&self) -> Vec<Response> {
        vec![]
    }
}

impl<T: Render> Render for anyhow::Result<T> {
    fn render_twitch(&self) -> Vec<Response> {
        match self {
            Ok(r) => r.render_twitch(),
            Err(e) => vec![Response::Problem(e.to_string())],
        }
    }

    fn render_discord(&self) -> Vec<Response> {
        match self {
            Ok(r) => r.render_discord(),
            Err(e) => vec![Response::Problem(e.to_string())],
        }
    }
}

impl<T: Render> Render for Option<T> {
    fn render_twitch(&self) -> Vec<Response> {
        self.as_ref().map(<_>::render_twitch).unwrap_or_default()
    }

    fn render_discord(&self) -> Vec<Response> {
        self.as_ref().map(<_>::render_discord).unwrap_or_default()
    }
}
