use super::CHANNEL_BUFFER_SIZE;
use std::sync::OnceLock;
use tokio::sync::mpsc::{self, Receiver, Sender, error::SendError};

static RUNNER_OP_TX: OnceLock<Sender<RunnerOperation>> = OnceLock::new();

pub fn runner_op_rx() -> Receiver<RunnerOperation> {
    let (tx, rx) = mpsc::channel::<RunnerOperation>(CHANNEL_BUFFER_SIZE);
    RUNNER_OP_TX.set(tx).unwrap();
    rx
}

pub async fn send_op(op: RunnerOperation) -> Result<(), SendError<RunnerOperation>> {
    let tx = RUNNER_OP_TX.get().expect("RUNNER_OP_TX is not initialized");
    tx.send(op).await
}

#[derive(Debug)]
pub enum RunnerOperation {
    UpdateView,
    Dispatch(diavolo::Action),
}
