use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Exit {
    #[serde(rename(serialize = "exit"))]
    pub value: ExitValue,
}

impl<'de> Deserialize<'de> for Exit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Exit {
            value: Deserialize::deserialize(deserializer)?,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ExitValue {
    PreEvaluation(String),
    ExitCode(u8),
}
