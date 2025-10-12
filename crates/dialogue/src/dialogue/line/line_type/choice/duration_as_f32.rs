use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;

pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = f32::deserialize(deserializer)?;
    Ok(Duration::from_secs_f32(seconds))
}

pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    duration.as_secs_f32().serialize(serializer)
}
