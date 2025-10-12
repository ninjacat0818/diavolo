pub mod actor_info;
pub mod args;
pub mod dialogue_name;
pub mod line;
pub mod lines;
pub mod location;
pub mod node;
pub mod nodes;
mod validate_impl;

use super::error::Error;
use actor_info::ActorInfo;
use args::Args;
use dialogue_name::DialogueName;
use nodes::*;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dialogue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<DialogueName>,
    #[serde(default, skip_serializing_if = "ActorInfo::is_skip_serializing")]
    pub actor: ActorInfo,
    #[serde(default, skip_serializing_if = "Args::is_empty")]
    pub args: Args,
    pub nodes: Nodes,
}

impl Dialogue {
    pub fn actor_num(&self) -> u8 {
        *self.actor.num
    }

    pub fn is_message_allowed(&self) -> bool {
        self.actor.is_actor_num_not_zero()
    }
}

impl FromStr for Dialogue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dialogue: Self = serde_yaml::from_str(s)?;
        dialogue.validate()?;
        Ok(dialogue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn serde_default() {
        let raw_dialogue = r#"
nodes:
  main: []
"#
        .trim_start();
        let default_dialogue = Dialogue::default();
        let deserialized: Dialogue = serde_yaml::from_str(raw_dialogue).unwrap();
        assert!(deserialized == default_dialogue);
        let serialized = serde_yaml::to_string(&default_dialogue).unwrap();
        assert_eq!(serialized, raw_dialogue);
    }

    #[test]
    fn explicit_actor_serializing() {
        let raw_dialogue = r#"
actor:
  num: 1
nodes:
  main: []
"#
        .trim_start();
        let deserialized: Dialogue = serde_yaml::from_str(raw_dialogue).unwrap();
        assert!(!deserialized.actor.is_skip_serializing());
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(raw_dialogue, serialized);
    }

    #[test]
    fn deserialize_no_main() {
        let raw_dialogue = r#"
actor:
  num: 1
nodes:
  foo:
  - message:
      en: Hello
    owner: 0
"#
        .trim_start();
        let result = raw_dialogue.parse::<Dialogue>();
        assert!(matches!(result, Err(Error::YamlError(_))));
    }

    #[test]
    fn deserialize_owner_out_of_range() {
        let raw_dialogue = r#"
nodes:
  main:
  - message: Hello
    owner: 1
"#
        .trim_start();
        let result = raw_dialogue.parse::<Dialogue>();
        assert!(matches!(result, Err(Error::OwnerOutOfRange { .. })));
    }

    #[test]
    fn deserialize_actor_zero() {
        let raw_dialogue = r#"
actor:
  num: 0
nodes:
  main:
  - message:
      en: This message should not be allowed because actor.num is 0.
    owner: 0
"#
        .trim_start();
        let result = raw_dialogue.parse::<Dialogue>();
        assert!(matches!(result, Err(Error::MessageNotAllowed { .. })));
    }

    #[test]
    fn deserialize() {
        let raw = r#"
  name: test dialogue script
  actor:
    num: 2
  args:
    key1: string
    key2: number
    key3: mut boolean
  nodes:
    main:
    - message:
        en: Hello
        ja: こんにちは
      options:
        emotion: happy
        speed: 1.0
    - choice:
        - indexed zero
        - indexed one
    - choice:
        foo:
          en: Foo choice
          ja: Foo選択肢
        bar:
          en: Bar choice
          ja: Bar選択肢
      options:
        message:
          texts:
            en: Please choose an option.
            ja: オプションを選んでください。
    foo_node:
    - message:
        en: Foo is selected.
        ja: Fooが選択されました。
    bar_node:
    - message:
        en: Bar is selected.
        ja: Barが選択されました。
"#;
        let _dialogue = raw.parse::<Dialogue>().unwrap();
    }
}
