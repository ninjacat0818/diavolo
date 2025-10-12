use super::super::runtimes::RuntimePath;
use super::CHANNEL_BUFFER_SIZE;
use crate::error;
use ws_message::client_message::{ClientRequest, Endpoint};

use std::sync::OnceLock;
use tokio::sync::mpsc::{self, Receiver, Sender, error::SendError};

static DIALOGUE_QUEUE_TX: OnceLock<Sender<DialogueQueueParams>> = OnceLock::new();

pub fn dialogue_queue_rx() -> Receiver<DialogueQueueParams> {
    let (tx, rx) = mpsc::channel::<DialogueQueueParams>(CHANNEL_BUFFER_SIZE);
    DIALOGUE_QUEUE_TX.set(tx).unwrap();
    rx
}

pub async fn send_dialogue_queue(
    params: DialogueQueueParams,
) -> Result<(), SendError<DialogueQueueParams>> {
    let tx = DIALOGUE_QUEUE_TX
        .get()
        .expect("DIALOGUE_QUEUE_TX is not initialized");
    tx.send(params).await
}

pub struct DialogueQueueParams {
    pub runtime_path: RuntimePath,
    pub actors: serde_json::Value,
    pub args: serde_json::Value,
    pub dialogue: diavolo::Dialogue,
}

impl DialogueQueueParams {
    pub async fn from_request(path: &str, client_request: ClientRequest) -> error::Result<Self> {
        let dialogue: diavolo::Dialogue = match client_request.endpoint {
            Endpoint::Url(url) => unimplemented!("Dialogue from URL is not implemented: {}", url),
            Endpoint::Path(path) => tokio::fs::read_to_string(path).await?.parse()?,
        };

        Ok(Self {
            runtime_path: RuntimePath::from(path),
            actors: client_request.actors,
            args: client_request.args,
            dialogue,
        })
    }
}
