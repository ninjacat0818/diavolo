use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Mutation,
    Terminated,
    Error,
}

impl Display for ServerMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).expect("Failed to serialize ServerMessage");
        write!(f, "{}", s)
    }
}

impl FromStr for ServerMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
