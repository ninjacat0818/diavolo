pub mod choice_texts;
pub mod duration_as_f32;

use super::Message;
pub use choice_texts::{ChoiceKey, ChoiceTexts};

use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Choice {
    #[serde(rename(serialize = "choice"))]
    pub texts: ChoiceTexts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChoiceOptions>,
}

impl Choice {
    pub fn has_message(&self) -> bool {
        self.options
            .as_ref()
            .and_then(|opts| opts.message.as_ref())
            .is_some()
    }

    pub fn message(&self) -> Option<&Message> {
        self.options.as_ref().and_then(|opts| opts.message.as_ref())
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ChoiceOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<ChoiceKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timeout>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Timeout(#[serde(with = "duration_as_f32")] pub std::time::Duration);

impl Deref for Timeout {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
