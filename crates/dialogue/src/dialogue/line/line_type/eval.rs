use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Eval {
    #[serde(rename(serialize = "eval"))]
    pub source: String,
}

impl<'de> Deserialize<'de> for Eval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Eval {
            source: Deserialize::deserialize(deserializer)?,
        })
    }
}

impl Deref for Eval {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
