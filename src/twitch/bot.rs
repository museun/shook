use std::sync::Arc;

use tokio_stream::StreamExt;

use crate::{
    binding::{Callable, Dispatch},
    message::{Message, MessageKind, TwitchMessage},
    render::Response,
    state::SharedState,
};

use super::{Connection, Privmsg};

pub struct Bot<const N: usize> {
    conn: Connection,
    state: SharedState,
    callables: [Arc<Callable>; N],
}

impl<const N: usize> Bot<N> {
    pub const fn new(conn: Connection, state: SharedState, callables: [Arc<Callable>; N]) -> Self {
        Self {
            conn,
            state,
            callables,
        }
    }

    pub async fn join(&mut self, channel: &str) -> anyhow::Result<()> {
        self.conn.write_raw(&format!("JOIN {channel}\r\n")).await
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        while let Ok(msg) = self.conn.read_privmsg().await {
            self.dispatch(msg).await?
        }
        Ok(())
    }

    async fn dispatch(&mut self, msg: Privmsg) -> anyhow::Result<()> {
        log::debug!("[{}]: {}", msg.user, msg.data);

        let msg = Message::new(
            TwitchMessage::from_pm(msg),
            MessageKind::Twitch,
            self.state.clone(),
        );

        let mut stream = Dispatch::new(&self.callables, std::convert::identity)
            .dispatch(&msg)
            .await;

        let channel = msg.as_twitch().unwrap().channel();
        let sender = msg.sender_name();

        while let Some(resp) = stream.next().await {
            for resp in resp.render_twitch() {
                let out = match resp {
                    Response::Say(msg) => {
                        format!("PRIVMSG {channel} :{msg}\r\n")
                    }
                    Response::Reply(msg) => {
                        format!("PRIVMSG {channel} :{sender} {msg}\r\n")
                    }
                    Response::Problem(msg) => {
                        format!("PRIVMSG {channel} :a problem occurred: {msg}\r\n")
                    }
                };
                self.conn.write_raw(&out).await?
            }
        }

        Ok(())
    }
}
