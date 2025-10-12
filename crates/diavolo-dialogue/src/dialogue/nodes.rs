use super::node::Node;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(transparent)]
pub struct Nodes(IndexMap<NodeKey, Node>);

impl<'de> Deserialize<'de> for Nodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: IndexMap<NodeKey, Node> = IndexMap::<NodeKey, Node>::deserialize(deserializer)?;
        if !map.contains_key(&NodeKey::main()) {
            return Err(serde::de::Error::custom("Missing 'main' node"));
        }
        Ok(Nodes(map))
    }
}

impl Deref for Nodes {
    type Target = IndexMap<NodeKey, Node>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Nodes {
    fn default() -> Self {
        let mut nodes = IndexMap::default();
        nodes.insert(NodeKey::main(), Node::default());
        Nodes(nodes)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct NodeKey(String);

impl NodeKey {
    pub fn main() -> Self {
        NodeKey("main".into())
    }

    pub fn uninitialized() -> Self {
        NodeKey("__uninitialized__".into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        let raw = r#"
main:
- message:
    en: Hello, world!
  owner: 0
foo:
- message:
    en: Hello, foo!
  owner: 0
bar:
- message:
    en: Hello, bar!
  owner: 0
"#
        .trim_start();

        let nodes: Nodes = serde_yaml::from_str(raw).unwrap();
        let serialized = serde_yaml::to_string(&nodes).unwrap();
        assert_eq!(raw, serialized);
    }
}
