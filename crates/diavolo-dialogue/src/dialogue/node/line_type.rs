pub(crate) mod choice;
pub(crate) mod confirm;
pub(crate) mod message;

pub(crate) mod lang_texts;

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

impl LineType {
    pub fn as_message(&self) -> Option<&Message> {
        match self {
            LineType::Message(message) => Some(message),
            LineType::Choice(choice) => choice
                .options
                .as_ref()
                .and_then(|opts| opts.message.as_ref()),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&Choice> {
        match self {
            LineType::Choice(choice) => Some(choice),
            _ => None,
        }
    }
}

impl Default for LineType {
    fn default() -> Self {
        LineType::Message(Message::default())
    }
}
