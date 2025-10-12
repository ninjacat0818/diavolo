use super::CHANNEL_BUFFER_SIZE;
use std::sync::OnceLock;
use tokio::sync::mpsc::{self, Receiver, Sender, error::SendError};

static VIEW_TX: OnceLock<Sender<diavolo::View<'static>>> = OnceLock::new();

pub fn view_rx() -> Receiver<diavolo::View<'static>> {
    let (tx, rx) = mpsc::channel::<diavolo::View<'static>>(CHANNEL_BUFFER_SIZE);
    VIEW_TX.set(tx).unwrap();
    rx
}

pub async fn send_view(
    view: diavolo::View<'static>,
) -> Result<(), SendError<diavolo::View<'static>>> {
    let tx = VIEW_TX.get().expect("VIEW_TX is not initialized");
    tx.send(view).await
}
