use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone)]
pub struct ArgVar {
    r#type: ArgType,
    mutable: bool,
}

impl ArgVar {
    pub fn new(r#type: ArgType, mutable: bool) -> Self {
        Self { r#type, mutable }
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }

    pub fn type_of(&self) -> &ArgType {
        &self.r#type
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum ArgType {
    String,
    Integer,
    Number,
    Bool,
}

impl Display for ArgVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.r#type {
            ArgType::String => "string",
            ArgType::Integer => "integer",
            ArgType::Number => "number",
            ArgType::Bool => "boolean",
        };
        let prefix = if self.is_mutable() { "mut " } else { "" };
        write!(f, "{}", format!("{prefix}{s}"))
    }
}

impl Serialize for ArgVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ArgVar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ArgVarVisitor;

        impl<'de> Visitor<'de> for ArgVarVisitor {
            type Value = ArgVar;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing an argument type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mutable = value.starts_with("mut ");
                let mut_erased = if mutable { &value[4..] } else { &value[..] };

                let r#type = match mut_erased {
                    "string" => Ok(ArgType::String),
                    "integer" => Ok(ArgType::Integer),
                    "number" => Ok(ArgType::Number),
                    "boolean" => Ok(ArgType::Bool),
                    other => Err(serde::de::Error::custom(format!(
                        "Unknown arg type: {}",
                        other
                    ))),
                }?;

                Ok(ArgVar::new(r#type, mutable))
            }
        }

        deserializer.deserialize_str(ArgVarVisitor)
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mut(bool);

impl Deref for Mut {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn serde() {
        let raw = "mut string\n";
        let deserialized: ArgVar = serde_yaml::from_str(raw).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(deserialized, ArgVar::new(ArgType::String, true));
        assert_eq!(serialized, raw);
    }
}
