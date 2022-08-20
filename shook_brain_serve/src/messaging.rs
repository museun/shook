use tokio::sync::{mpsc::Sender, oneshot};

use crate::request;

#[derive(Clone)]
pub struct Messaging {
    tx: Sender<(Request, oneshot::Sender<Response>)>,
}

impl Messaging {
    pub const fn new(tx: Sender<(Request, oneshot::Sender<Response>)>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, req: Request) -> Response {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send((req, tx)).await;
        rx.await.unwrap()
    }
}

#[derive(Debug)]
pub enum Response {
    Generated { data: String },
    Error { error: anyhow::Error },
    Nothing,
}

pub enum Request {
    Train { data: String },
    Generate { opts: request::Generate },
    Save,
    ForceSave,
}
