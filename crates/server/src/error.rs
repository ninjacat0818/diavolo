use super::channels::DialogueQueueParams;
use tokio::sync::mpsc::error::SendError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Boxed error: {0}")]
    BoxedError(#[from] Box<dyn std::error::Error>),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Dialogue queue params send error: {0}")]
    DialogueQueueParamsSendError(#[from] SendError<DialogueQueueParams>),
    #[error("View send error: {0}")]
    ViewSendError(#[from] SendError<diavolo::View<'static>>),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Dialogue error: {0}")]
    DialogueError(#[from] diavolo::dialogue::Error),

    #[error("Server error: {0}")]
    ServerError(#[from] ServerError),
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Runtime not found")]
    RuntimeNotFound,
}
