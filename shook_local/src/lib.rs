use std::{net::SocketAddr, sync::Arc};

use shook_core::{
    prelude::{GlobalState, Message, SharedCallable},
    render::{dispatch_and_render, RenderFlavor, Response},
};

use shook_twitch as twitch;

use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Clone)]
pub struct LocalPort(SocketAddr);

impl std::fmt::Display for LocalPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn create_bot<const N: usize>(
    state: GlobalState,
    handlers: [SharedCallable; N],
) -> anyhow::Result<()> {
    let listener = TcpListener::bind("localhost:0").await?;
    let addr = listener.local_addr()?;
    log::info!("local server is listening on: {addr}");
    state.insert(LocalPort(addr)).await;

    loop {
        if let Ok((client, addr)) = listener.accept().await {
            log::debug!("got client: {addr}");
            tokio::spawn(handle(client, state.clone(), handlers.clone()));
        }
    }
}

async fn handle<const N: usize>(
    mut client: TcpStream,
    state: GlobalState,
    handlers: [SharedCallable; N],
) {
    let (read, mut write) = client.split();

    let tags = twitch::Tags::parse(&mut "@badges=broadcaster/1 ").unwrap();
    let (user, channel) = (Arc::from("museun"), Arc::from("#museun"));

    let mut reader = BufReader::new(read).lines();
    while let Ok(Some(line)) = reader.next_line().await {
        let msg = Message::new(
            twitch::Message::from_pm(twitch::Privmsg {
                tags: tags.clone(),
                user: Arc::clone(&user),
                target: Arc::clone(&channel),
                data: line.into(),
            }),
            state.clone(),
        );

        for resp in dispatch_and_render(&handlers, &msg, RenderFlavor::Twitch).await {
            let (kind, out) = match resp {
                Response::Say(msg) => ("say", msg),
                Response::Reply(msg) => ("reply", msg),
                Response::Problem(msg) => ("problem", msg),
            };
            let out = format!("{kind} -> {out}");
            let _ = send_message(&mut write, Line::Message { data: &out }).await;
        }
        let _ = send_message(&mut write, Line::None).await;
    }
}

#[derive(serde::Serialize)]
enum Line<'a> {
    Message { data: &'a str },
    None,
}

async fn send_message(
    mut w: impl AsyncWrite + Send + Unpin + Sized,
    msg: impl serde::Serialize,
) -> tokio::io::Result<()> {
    w.write_all(&serde_json::to_vec(&msg).expect("valid json"))
        .await?;
    w.write_all(b"\n").await?;
    w.flush().await
}
