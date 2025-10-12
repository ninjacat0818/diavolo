pub mod line;
pub mod line_type;

use line::Line;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Node(Lines);

impl Deref for Node {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Lines(Vec<Line>);

impl Default for Lines {
    fn default() -> Self {
        Lines(vec![Line::default()])
    }
}

impl Deref for Lines {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}