use std::future::Future;

use tokio_stream::{Stream, StreamExt};

use crate::{
    prelude::Message,
    render::{BoxedRender, Render},
};

use super::CallableFn;

pub struct Dispatch<'a, C> {
    seq: &'a [C],
}

impl<'a, C> Dispatch<'a, C>
where
    C: CallableFn + Clone,
    <<C as CallableFn>::Out as Future>::Output: Render,
{
    pub const fn new(seq: &'a [C]) -> Self {
        Self { seq }
    }

    pub fn dispatch(self, msg: &Message) -> impl Stream<Item = BoxedRender> {
        let (tx, rx) = tokio::sync::mpsc::channel(std::cmp::max(1, self.seq.len()));
        for callable in self.seq.iter().map(|c| c.clone()) {
            let tx = tx.clone();
            let msg = msg.clone();
            let _ = tokio::spawn(async move {
                let render = callable.call(msg).await;
                let _ = tx.send(render.boxed()).await;
            });
        }
        drop(tx);
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }

    pub async fn into_render(self, msg: &Message) -> impl Render
    where
        Self: 'a,
    {
        let mut stream = self.dispatch(msg);
        let mut out = vec![];
        while let Some(el) = stream.next().await {
            out.push(el);
        }
        out
    }
}
