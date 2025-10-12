use super::lines::Lines;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Node(Lines);

impl Deref for Node {
    type Target = Lines;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
