use std::{future::Future, sync::Arc};

use crate::{
    args::{Arguments, Match},
    prelude::Message,
    render::{BoxedRender, Render, RenderFlavor, Response},
};

use super::Command;

pub async fn wrap<F>(
    mut msg: Message,
    cmd: Arc<Command>,
    func: impl Fn(Message) -> F + Send + Sync,
) -> BoxedRender
where
    F: Future + Send,
    F::Output: Render + Send + 'static,
{
    enum MatchError {
        Required { usage: String },
        NoMatch { usage: String },
    }

    impl Render for MatchError {
        fn render(&self, flavor: RenderFlavor) -> Vec<Response> {
            // TODO make this better for discord
            let data = match self {
                Self::Required { usage } => format!("an argument is required: {usage}"),
                Self::NoMatch { usage } => format!("invalid arguments: {usage}"),
            };
            data.render(flavor)
        }
    }

    fn check_command(cmd: &Command, msg: &Message) -> bool {
        [&*cmd.command]
            .into_iter()
            .chain(cmd.aliases.iter().map(|c| &**c))
            .fold(false, |ok, c| ok ^ msg.match_command(c))
    }

    if !check_command(&cmd, &msg) {
        return ().boxed();
    }

    // TODO just do split_at
    let head = std::cmp::min(msg.command().len() + 1, msg.data().len());
    let input = &msg.data()[head..];

    if let Some(example) = &cmd.example {
        let args = match example.extract(input) {
            Match::Required => {
                let usage = cmd.command.to_string();
                return MatchError::Required { usage }.boxed();
            }

            Match::NoMatch => {
                let usage = cmd.command.to_string();
                return MatchError::NoMatch { usage }.boxed();
            }
            Match::Match(map) => Arguments { map },
            Match::Exact => Arguments::default(),
        };
        msg.get_args().replace(args);
    };

    func(msg).await.boxed()
}
