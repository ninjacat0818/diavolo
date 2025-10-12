#[derive(thiserror::Error, Debug)]
pub enum DialogueParseError {
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
}

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("Owner {owner} exceeds maximum {max_owner} at {location}")]
    OwnerOutOfRange {
        owner: u8,
        max_owner: u8,
        location: Location,
    },

    #[error("actor.num is 0, so no message is allowed at {location}")]
    MessageNotAllowed { location: Location },

    #[error("Referenced node '{referenced_node}' not found at {location}")]
    NodeNotFound {
        referenced_node: String,
        location: String,
    },
}

use super::dialogue::nodes::NodeKey;

#[derive(Debug, Clone)]
pub struct Location {
    node: NodeKey,
    line_idx: usize,
}

impl Location {
    pub fn new(node: NodeKey, line_idx: usize) -> Self {
        Self { node, line_idx }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node '{}' line {}", self.node, self.line_idx)
    }
}
