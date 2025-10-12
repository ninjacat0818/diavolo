use super::line::Line;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Lines(Vec<Line>);

impl Deref for Lines {
    type Target = Vec<Line>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
