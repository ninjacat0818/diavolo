mod duration_as_f32;

use super::LangTexts;
use super::Message;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Choice {
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
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ChoiceTexts(IndexMap<ChoiceKey, LangTexts>);

impl Deref for ChoiceTexts {
    type Target = IndexMap<ChoiceKey, LangTexts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct ChoiceKey(String);

impl Deref for ChoiceKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
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
