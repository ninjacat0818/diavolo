use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LineIf(String);

impl AsRef<str> for LineIf {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for LineIf {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
