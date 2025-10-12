use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LineId(String);

impl LineId {
    pub fn unique() -> Self {
        LineId(nanoid::nanoid!())
    }
}

impl From<String> for LineId {
    fn from(s: String) -> Self {
        LineId(s)
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
