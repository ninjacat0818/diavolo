use super::CHANNEL_BUFFER_SIZE;
use std::sync::OnceLock;
use tokio::sync::mpsc::{self, Receiver, Sender, error::SendError};

static RUNNER_STATE_INFO_TX: OnceLock<Sender<RunnerStateInfo>> = OnceLock::new();

pub fn runner_state_info_rx() -> Receiver<RunnerStateInfo> {
    let (tx, rx) = mpsc::channel::<RunnerStateInfo>(CHANNEL_BUFFER_SIZE);
    RUNNER_STATE_INFO_TX.set(tx).unwrap();
    rx
}

pub async fn send_runner_state_info(
    info: RunnerStateInfo,
) -> Result<(), SendError<RunnerStateInfo>> {
    let tx = RUNNER_STATE_INFO_TX
        .get()
        .expect("RUNNER_STATE_INFO_TX is not initialized");
    tx.send(info).await
}

pub enum RunnerStateInfo {
    Mutation,
    Terminated,
    Error,
}
