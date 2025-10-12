use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Args(HashMap<ArgName, ArgType>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ArgName(String);

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ArgType {
    String(Mut),
    Int(Mut),
    Float(Mut),
    Bool(Mut),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mut(bool);
