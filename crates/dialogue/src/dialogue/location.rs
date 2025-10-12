use super::line::Text;

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug)]
pub struct Location {
    pub node_key: NodeKey,
    pub line_position: LinePosition,
}

impl Location {
    pub fn uninitialized(node_key: NodeKey) -> Self {
        Location {
            node_key,
            line_position: LinePosition::uninitialized(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct NodeKey(String);

impl NodeKey {
    pub fn main() -> Self {
        NodeKey("main".into())
    }
}

impl std::borrow::Borrow<str> for NodeKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Text> for NodeKey {
    fn from(text: Text) -> Self {
        text.as_str().into()
    }
}

impl From<String> for NodeKey {
    fn from(s: String) -> Self {
        NodeKey(s)
    }
}

impl From<&str> for NodeKey {
    fn from(s: &str) -> Self {
        NodeKey(s.into())
    }
}

impl Display for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinePosition(usize);

impl LinePosition {
    pub fn uninitialized() -> Self {
        LinePosition(usize::MAX)
    }
}

impl Deref for LinePosition {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for LinePosition {
    fn from(value: usize) -> Self {
        LinePosition(value)
    }
}
