use super::LangTexts;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Message {
    pub texts: LangTexts,
    pub owner: Owner,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MessageOptions>,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Owner(u8);

impl Deref for Owner {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MessageOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<TypingSpeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listeners: Option<Listeners>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Listeners(BTreeSet<u8>);

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypingSpeed(f32);

impl std::ops::Deref for TypingSpeed {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<f32> for TypingSpeed {
    fn from(value: f32) -> Self {
        TypingSpeed(value)
    }
}

impl Default for TypingSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FontProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
}
