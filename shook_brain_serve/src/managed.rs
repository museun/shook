use std::{path::PathBuf, time::Duration};

use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    time::Instant,
};

use crate::{
    messaging::{Request, Response},
    request::Generate,
    BrainExt,
};
use shook_markov::Brain;

pub struct ManagedBrain {
    brain: Brain,
    last_save: Instant,
    path: PathBuf,
    recv: Receiver<(Request, oneshot::Sender<Response>)>,
    timeout: Duration,
    save_duration: Duration,
}

impl ManagedBrain {
    pub fn spawn(
        brain: Brain,
        path: impl Into<PathBuf>,
        timeout: Duration,
        save_duration: Duration,
    ) -> Sender<(Request, oneshot::Sender<Response>)> {
        let (tx, recv) = channel(16);
        let mut this = Self {
            brain,
            last_save: Instant::now(),
            path: path.into(),
            recv,
            timeout,
            save_duration,
        };

        let _handle = std::thread::spawn(move || {
            while let Some((msg, out)) = this.recv.blocking_recv() {
                match msg {
                    Request::Train { data } => this.handle_train(&data, out),
                    Request::Generate { opts } => this.handle_generate(opts, out),
                    Request::Save => this.handle_save(out),
                    Request::ForceSave => this.handle_force_save(out),
                }
            }

            log::warn!("end of managed brain loop");
        });

        tx
    }

    fn handle_train(&mut self, data: &str, out: oneshot::Sender<Response>) {
        self.brain.train(data);
        let _ = out.send(Response::Nothing);
    }

    fn handle_generate(&self, opts: Generate, out: oneshot::Sender<Response>) {
        let _ = match self
            .brain
            .generate(opts.min, opts.max, opts.query.as_deref(), self.timeout)
        {
            Some(data) => out.send(Response::Generated { data }),
            None => {
                let error = anyhow::anyhow!("could not generate data");
                out.send(Response::Error { error })
            }
        };
    }

    fn handle_save(&mut self, out: oneshot::Sender<Response>) {
        if self.last_save.elapsed() >= self.save_duration {
            return self.handle_force_save(out);
        }
        let _ = out.send(Response::Nothing);
    }

    fn handle_force_save(&mut self, out: oneshot::Sender<Response>) {
        if let Err(error) = self.brain.save(&self.path) {
            let _ = out.send(Response::Error { error });
            return;
        }
        self.last_save = Instant::now();
        let _ = out.send(Response::Nothing);
    }
}
