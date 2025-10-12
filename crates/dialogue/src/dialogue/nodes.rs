use super::location::NodeKey;
use super::node::Node;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
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
