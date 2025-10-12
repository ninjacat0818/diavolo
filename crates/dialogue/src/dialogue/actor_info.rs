use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ActorInfo {
    pub num: ActorNum,
}

impl ActorInfo {
    pub(crate) fn is_owner_in_range(&self, owner: u8) -> bool {
        owner <= self.max_owner()
    }

    pub(crate) fn max_owner(&self) -> u8 {
        *self.num - 1
    }

    pub(crate) fn is_actor_num_not_zero(&self) -> bool {
        *self.num > 0
    }

    pub(crate) fn is_skip_serializing(&self) -> bool {
        self.num.is_default()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ActorNum {
    value: u8,
    #[serde(skip)]
    default: bool,
}

impl ActorNum {
    fn is_default(&self) -> bool {
        self.default
    }
}

impl Default for ActorNum {
    fn default() -> Self {
        ActorNum {
            value: 1,
            default: true,
        }
    }
}

impl Deref for ActorNum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
