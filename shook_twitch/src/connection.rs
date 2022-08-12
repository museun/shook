use std::sync::Arc;

use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
};

use super::{parser, tags::Tags, types::Registration, Identity, Privmsg};

pub struct Connection {
    stream: BufStream<TcpStream>,
    buf: String,
}

impl Connection {
    pub async fn connect(addr: &str, reg: Registration<'_>) -> anyhow::Result<(Identity, Self)> {
        let mut stream = TcpStream::connect(addr).await?;

        for cap in [
            "CAP REQ :twitch.tv/membership\r\n",
            "CAP REQ :twitch.tv/tags\r\n",
            "CAP REQ :twitch.tv/commands\r\n",
        ] {
            stream.write_all(cap.as_bytes()).await?;
        }
        stream.flush().await?;

        let Registration { name, pass, .. } = reg;
        for reg in [format!("PASS {pass}\r\n"), format!("NICK {name}\r\n")] {
            stream.write_all(reg.as_bytes()).await?;
        }
        stream.flush().await?;

        let mut stream = BufStream::new(stream);
        let mut buf = String::with_capacity(1024);
        let identity = Self::wait_for_ready(name, &mut buf, &mut stream).await?;
        buf.clear();

        Ok((identity, Self { stream, buf }))
    }

    pub async fn write_raw(&mut self, data: &str) -> anyhow::Result<()> {
        log::trace!(target:"shook::twitch","-> {}", data.escape_debug());
        self.stream.write_all(data.as_bytes()).await?;
        if !data.ends_with('\n') {
            self.stream.write_all(b"\r\n").await?;
        }
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn read_privmsg(&mut self) -> anyhow::Result<Privmsg> {
        loop {
            self.buf.clear();

            let n = self.stream.read_line(&mut self.buf).await?;
            let line = &self.buf[..n];

            let (tags, prefix, cmd, args, data) = parser::parse(line);
            let prefix = prefix.map(Arc::<str>::from);
            let data = data.map(Arc::<str>::from);

            match cmd {
                "PING" => {
                    let resp = format!("PONG :{}\r\n", data.unwrap());
                    self.stream.write_all(resp.as_bytes()).await?;
                    self.stream.flush().await?;
                }
                "ERROR" => anyhow::bail!("error: {:?}", data),
                "PRIVMSG" => {
                    return Ok(Privmsg {
                        tags,
                        user: prefix.expect("prefix attached"),
                        target: args[0].into(),
                        data: data.expect("malformed message"),
                    });
                }
                _ => {}
            }
        }
    }

    async fn wait_for_ready(
        default_name: &str,
        buf: &mut String,
        stream: &mut BufStream<TcpStream>,
    ) -> anyhow::Result<Identity> {
        loop {
            let n = stream.read_line(buf).await?;
            if n == 0 {
                anyhow::bail!("unexpected eof")
            }

            let mut raw = &buf[..n - 2];

            let tags = raw
                .starts_with('@')
                .then(|| Tags::parse(&mut raw))
                .flatten()
                .unwrap_or_default();

            match raw.split_once(' ') {
                Some(("PING", tail)) => {
                    let token = tail
                        .rsplit_terminator(':')
                        .next()
                        .with_context(|| "PING must have a token")?;
                    let out = format!("PONG :{token}\r\n");
                    stream.write_all(out.as_bytes()).await?;
                }
                Some((.., "GLOBALUSERSTATE")) => {
                    let name = tags.get("display-name").unwrap_or(default_name).into();
                    let user_id = tags.get_parsed("user-id")?;
                    let identity = Identity { name, user_id };
                    return Ok(identity);
                }
                Some(("ERROR", tail)) => anyhow::bail!("{tail}"),
                _ => {}
            }

            buf.clear();
        }
    }
}
