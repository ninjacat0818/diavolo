pub mod actor_info;
pub mod args;
pub mod dialogue_name;
pub mod node;
pub mod nodes;

use actor_info::ActorInfo;
use args::Args;
use dialogue_name::DialogueName;
use nodes::*;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dialogue {
    pub name: DialogueName,
    pub actor: ActorInfo,
    pub args: Args,
    pub nodes: Nodes,
}

impl FromStr for Dialogue {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        let raw = r#"
  name: "test dialogue script"
  actor:
    num: 1
  inputs:
    key1: String
    key2: Number
    key3: Boolean
  nodes:
    main:
    - message:
        en: Hello
        ja: こんにちは
      owner: 0
      options:
        emotion: happy
        speed: 1.0
    - choice:
        foo:
          en: Foo choice
          ja: Foo選択肢
        bar:
          en: Bar choice
          ja: Bar選択肢
      options:
        message:
          owner: 0
          texts:
            en: Please choose an option.
            ja: オプションを選んでください。
    foo_node:
    - message:
        en: Foo is selected.
        ja: Fooが選択されました。
      owner: 0
    bar_node:
    - message:
        en: Bar is selected.
        ja: Barが選択されました。
      owner: 0
"#;
        let _dialogue: Dialogue = serde_yaml::from_str(raw).unwrap();
    }
}
