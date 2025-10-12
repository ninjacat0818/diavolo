use serde::{Deserialize, Serialize};

use super::texts::Text;

#[derive(Debug, PartialEq, Clone, Default, Serialize)]
pub struct Call {
    #[serde(rename(serialize = "call"))]
    pub pre_evaluation_node_key: Text,
}

impl<'de> Deserialize<'de> for Call {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pre_evaluation_node_key: Text = Deserialize::deserialize(deserializer)?;
        Ok(Call { pre_evaluation_node_key })
    }
}