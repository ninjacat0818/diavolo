use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Return {
    #[serde(rename(serialize = "return"))]
    pub pre_evaluation_value: ReturnValue,
}

impl<'de> Deserialize<'de> for Return {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Return {
            pre_evaluation_value: Deserialize::deserialize(deserializer)?,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnValue(String);

impl AsRef<str> for ReturnValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ReturnValue {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for ReturnValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0.as_str() {
            "undefined" => serializer.serialize_unit(),
            "null" => serializer.serialize_none(),
            "true" => serializer.serialize_bool(true),
            "false" => serializer.serialize_bool(false),
            _ => {
                if let Ok(n) = self.0.parse::<f64>() {
                    serializer.serialize_f64(n)
                } else {
                    serializer.serialize_str(&self.0)
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for ReturnValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value {
            String(String),
            Number(f64),
            Bool(bool),
            Null,
        }

        let value: Value = Deserialize::deserialize(deserializer)?;

        let pre_evaluation_value = match value {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "undefined".to_string(),
        };

        Ok(ReturnValue(pre_evaluation_value))
    }
}
