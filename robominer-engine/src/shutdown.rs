//! Shared ctrl-c shutdown watch for long-running engine worker loops.

pub(crate) struct ShutdownSignal {
    receiver: tokio::sync::watch::Receiver<bool>,
}

impl ShutdownSignal {
    pub(crate) fn requested(&self) -> bool {
        *self.receiver.borrow()
    }

    pub(crate) async fn wait(&mut self) {
        if self.requested() {
            return;
        }

        let _ = self.receiver.changed().await;
    }
}

pub(crate) fn shutdown_signal() -> ShutdownSignal {
    let (sender, receiver) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("failed to listen for shutdown signal: {error}");
            return;
        }

        let _ = sender.send(true);
    });

    ShutdownSignal { receiver }
}
