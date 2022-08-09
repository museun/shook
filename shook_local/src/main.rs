#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use gumdrop::Options;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, gumdrop::Options)]
struct Args {
    /// prints the help message
    help: bool,

    /// port to connect to
    #[options(meta = "<PORT>")]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let Args { port, .. } = Args::parse_args_default_or_exit();

    let stream = tokio::net::TcpStream::connect(format!("localhost:{port}")).await?;
    let (read, mut write) = stream.into_split();
    let mut read = BufReader::new(read).lines();

    let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    let mut out = tokio::io::stdout();

    'outer: loop {
        out.write_all(b"> ").await?;
        out.flush().await?;

        let line = match stdin.next_line().await? {
            Some(line) if !line.is_empty() => line,
            Some(..) => continue,
            None => break,
        };

        write.write_all(line.as_bytes()).await?;
        write.write_all(b"\n").await?;
        write.flush().await?;

        'inner: loop {
            let resp = match read.next_line().await? {
                Some(resp) => resp,
                None => break 'outer,
            };

            match serde_json::from_str(&resp).expect("valid json") {
                Line::Message { data } => eprintln!("{data}"),
                Line::None => break 'inner,
            }
        }
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
enum Line {
    Message { data: String },
    None,
}
