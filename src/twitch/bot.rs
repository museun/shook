use crate::{
    callable::SharedCallable,
    message::{Message, MessageKind},
    render::{dispatch_and_render, RenderFlavor, Response},
    state::GlobalState,
};

use super::{Connection, Message as TwitchMessage, Privmsg};

pub struct Bot<const N: usize> {
    conn: Connection,
    state: GlobalState,
    callables: [SharedCallable; N],
}

impl<const N: usize> Bot<N> {
    pub const fn new(conn: Connection, state: GlobalState, callables: [SharedCallable; N]) -> Self {
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
        log::debug!(target:"shook::twitch","[{}] {}: {}", msg.target, msg.user, msg.data);

        let msg = Message::new(
            TwitchMessage::from_pm(msg),
            MessageKind::Twitch,
            self.state.clone(),
        );

        let channel = msg.as_twitch().unwrap().channel();
        let sender = msg.sender_name();

        for resp in dispatch_and_render(&self.callables, &msg, RenderFlavor::Twitch).await {
            let out = match resp {
                Response::Say(msg) => {
                    format!("PRIVMSG {channel} :{msg}\r\n")
                }
                Response::Reply(msg) => {
                    format!("PRIVMSG {channel} :{sender}: {msg}\r\n")
                }
                Response::Problem(msg) => {
                    format!("PRIVMSG {channel} :a problem occurred: {msg}\r\n")
                }
            };
            self.conn.write_raw(&out).await?
        }

        Ok(())
    }
}
