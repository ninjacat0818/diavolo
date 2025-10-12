use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ActorInfo {
    num: u8,
}
