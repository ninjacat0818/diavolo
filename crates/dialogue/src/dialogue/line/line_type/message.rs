use super::texts::Texts;

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::collections::BTreeSet;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize)]
pub struct Message {
    pub texts: Texts,
    #[serde(default)]
    pub owner: Owner,
    pub options: Option<MessageOptions>,
    #[serde(skip)]
    pub(crate) is_options: bool,
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Message", 3)?;

        state.serialize_field(
            self.is_options.then_some("texts").unwrap_or("message"),
            &self.texts,
        )?;
        if !self.owner.is_skip_serializing() {
            state.serialize_field("owner", &self.owner)?;
        }
        if let Some(options) = &self.options {
            state.serialize_field("options", options)?;
        }

        state.end()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Owner {
    pub(crate) id: u8,
    #[serde(skip)]
    default: bool,
}

impl Owner {
    pub(super) fn is_skip_serializing(&self) -> bool {
        self.default
    }
}

impl Default for Owner {
    fn default() -> Self {
        Self {
            id: 0,
            default: true,
        }
    }
}

impl Deref for Owner {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MessageOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<TypingSpeedFactor>,
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
pub struct TypingSpeedFactor(f32);

impl std::ops::Deref for TypingSpeedFactor {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<f32> for TypingSpeedFactor {
    fn from(value: f32) -> Self {
        TypingSpeedFactor(value)
    }
}

impl Default for TypingSpeedFactor {
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
