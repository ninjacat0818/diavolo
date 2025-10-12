mod arg_var;

pub use arg_var::{ArgType, ArgVar};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Args(HashMap<ArgName, ArgVar>);

impl Deref for Args {
    type Target = HashMap<ArgName, ArgVar>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Args {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Args {
    pub fn to_json_schema(&self) -> Value {
        let mut properties = Map::new();

        for (name, vars) in self.iter() {
            let property_value = match vars.type_of() {
                ArgType::String => json!({"type": "string"}),
                ArgType::Integer => json!({"type": "integer"}),
                ArgType::Number => json!({"type": "number"}),
                ArgType::Bool => json!({"type": "boolean"}),
            }
            .as_object()
            .unwrap()
            .clone();

            properties.insert(name.as_str().into(), property_value.into());
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": self.keys().map(|k| k.as_str()).collect::<Vec<&str>>(),
            "additionalProperties": false
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ArgName(String);

impl From<&str> for ArgName {
    fn from(s: &str) -> Self {
        ArgName(s.to_owned())
    }
}

impl Deref for ArgName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
