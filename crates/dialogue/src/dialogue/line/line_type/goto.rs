use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Goto {
    #[serde(rename(serialize = "goto"))]
    pub pre_evaluation_value: GotoValue,
}

impl<'de> Deserialize<'de> for Goto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Goto {
            pre_evaluation_value: Deserialize::deserialize(deserializer)?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GotoValue(String);

impl Deref for GotoValue {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for GotoValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Ok(n) = self.parse::<usize>() {
            serializer.serialize_i32(n as i32)
        } else {
            serializer.serialize_str(&self)
        }
    }
}

impl<'de> Deserialize<'de> for GotoValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            String(String),
            Number(usize),
        }

        let value: Value = Deserialize::deserialize(deserializer)?;

        let pre_evaluation_value = match value {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
        };

        Ok(GotoValue(pre_evaluation_value))
    }
}
