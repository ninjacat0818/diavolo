use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ActorInfo {
    pub num: ActorNum,
}

impl ActorInfo {
    pub(super) fn is_owner_in_range(&self, owner: u8) -> bool {
        owner <= self.max_owner()
    }

    pub(super) fn max_owner(&self) -> u8 {
        *self.num - 1
    }

    pub(super) fn is_actor_num_not_zero(&self) -> bool {
        *self.num > 0
    }

    pub(super) fn is_default(&self) -> bool {
        self == &ActorInfo::default()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ActorNum(u8);

impl Deref for ActorNum {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
