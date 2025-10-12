use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum ClientMessage {
    Request(ClientRequest),
    Mutation,
    Cancel,
}

impl Display for ClientMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).expect("Failed to serialize ClientMessage");
        write!(f, "{}", s)
    }
}

impl TryFrom<serde_json::Value> for ClientMessage {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl FromStr for ClientMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientRequest {
    pub endpoint: Endpoint,
    pub args: serde_json::Value,
    pub actors: serde_json::Value,
}

pub enum Endpoint {
    Url(url::Url),
    Path(std::path::PathBuf),
}

impl From<&str> for Endpoint {
    fn from(s: &str) -> Self {
        if let Ok(url) = url::Url::parse(s) {
            Endpoint::Url(url)
        } else {
            Endpoint::Path(std::path::PathBuf::from(s))
        }
    }
}

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Endpoint::Url(url) => serializer.serialize_str(url.as_str()),
            Endpoint::Path(path) => serializer.serialize_str(path.to_str().unwrap_or_default()),
        }
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Ok(url) = url::Url::parse(&s) {
            Ok(Endpoint::Url(url))
        } else {
            Ok(Endpoint::Path(std::path::PathBuf::from(s)))
        }
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn serde_client_message() {
        let deserialized: ClientMessage = serde_json::json!({
            "type": "request",
            "endpoint": "/dialogue_script.yml",
            "actors": {},
            "args": {
                "x": 42,
                "y": "Hello, Diavolo!"
            },
        })
        .try_into()
        .unwrap();

        deserialized.to_string().parse::<ClientMessage>().unwrap();
    }
}
