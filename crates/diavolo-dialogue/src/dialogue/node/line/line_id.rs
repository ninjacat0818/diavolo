use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LineId(String);

impl LineId {
    pub fn new_unique() -> Self {
        LineId(nanoid::nanoid!())
    }
}

impl AsRef<str> for LineId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for LineId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
