mod choice;
mod confirm;
mod message;

mod lang_texts;

pub use choice::*;
pub use confirm::*;
pub use lang_texts::*;
pub use message::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    Message(Message),
    Choice(Choice),
    Confirm(Confirm),
    // Input,
    // Eval,
    // Event,
    // Call,
    // Use,
}

impl Default for LineType {
    fn default() -> Self {
        LineType::Message(Message::default())
    }
}