use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DialogueName(String);
