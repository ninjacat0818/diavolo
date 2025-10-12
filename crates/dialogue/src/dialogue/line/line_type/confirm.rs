use super::Texts;
use super::message::{Message, MessageOptions};

use serde::{Deserialize, Serialize, ser::SerializeMap};

#[derive(Debug, PartialEq, Clone)]
pub struct Confirm {
    pub message: Message,
    pub options: Option<ConfirmOptions>,
}

impl Serialize for Confirm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(3))?;

        state.serialize_entry("confirm", &self.message.texts)?;

        if !self.message.owner.is_skip_serializing() {
            state.serialize_entry("owner", &self.message.owner)?;
        }

        if let Some(options) = &self.options {
            #[derive(Serialize)]
            struct Options<'a> {
                response: &'a Option<ConfirmResponse>,
                message: &'a Option<MessageOptions>,
            }

            let opts = Options {
                response: &options.response,
                message: &self.message.options,
            };

            state.serialize_entry("options", &opts)?;
        }

        state.end()
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ConfirmOptions {
    pub response: Option<ConfirmResponse>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ConfirmResponse {
    pub yes: Texts,
    pub no: Texts,
}
