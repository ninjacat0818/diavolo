pub mod line_id;
pub mod line_if;
pub mod line_type;

pub use line_id::*;
pub use line_if::*;
pub use line_type::*;

use serde::{Deserialize, Serialize, de};

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Line {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<LineId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<LineIf>,
    #[serde(flatten)]
    pub r#type: LineType,
}

impl<'de> Deserialize<'de> for Line {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(MapVisitor)
    }
}

use std::collections::HashMap;
type DiscriminatorEntriesMap<'a> = HashMap<&'a str, HashMap<String, serde_json::Value>>;
struct MapVisitor;

impl<'de> serde::de::Visitor<'de> for MapVisitor {
    type Value = Line;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map representing a Line")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::MapAccess<'de>,
    {
        let mut id: Option<LineId> = None;
        let mut r#if: Option<LineIf> = None;
        let mut options: Option<serde_json::Value> = None;
        let mut discriminator: Option<Discriminator> = None;

        // Currently only 'message' has additional entries ('owner')
        let mut dem: DiscriminatorEntriesMap = HashMap::from([("message", HashMap::new())]);

        while let Some(key) = map.next_key::<String>()? {
            match key.as_ref() {
                k @ ("message" | "choice" | "confirm" | "eval" | "goto" | "call" | "return"
                | "exit") => {
                    if options.is_some() {
                        match k {
                            "message" | "choice" | "confirm" => {}
                            _ => {
                                let msg = format!("'{k}' cannot be used with 'options'");
                                return Err(de::Error::custom(msg));
                            }
                        }
                    }

                    if discriminator.is_some() {
                        let msg = format!("mutually exclusive keys present: {k}",);
                        return Err(de::Error::custom(msg));
                    }
                    let conflicting_entry =
                        dem.iter().filter(|(n, _)| **n != k).find_map(|(n, e)| {
                            (!e.is_empty()).then(|| (*n, e.keys().next().unwrap().as_str()))
                        });
                    if let Some((name, key)) = conflicting_entry {
                        let msg = format!("'{key}' can only be used with: {name}");
                        return Err(de::Error::custom(msg));
                    }
                    let value = map.next_value::<serde_json::Value>()?;
                    discriminator = Some(Discriminator::from((k, value)));
                }
                k @ "owner" => {
                    if discriminator.is_some()
                        && !matches!(discriminator, Some(Discriminator::Message(_)))
                    {
                        let msg = format!("'{k}' can only be used with 'message'");
                        return Err(de::Error::custom(msg));
                    }
                    let entries = dem.get_mut("message").unwrap();
                    if entries.contains_key(k) {
                        return Err(de::Error::custom(format!("duplicate key found: {k}")));
                    }
                    entries.insert(k.to_owned(), map.next_value::<serde_json::Value>()?);
                }
                "id" => {
                    id = id.map_or_else(
                        || Ok(Some(map.next_value::<LineId>()?)),
                        |_| Err(de::Error::custom("duplicate 'id' key found")),
                    )?;
                }
                "if" => {
                    r#if = r#if.map_or_else(
                        || Ok(Some(map.next_value::<LineIf>()?)),
                        |_| Err(de::Error::custom("duplicate 'if' key found")),
                    )?;
                }
                "options" => {
                    if let Some(discriminator) = discriminator.as_ref() {
                        match discriminator {
                            Discriminator::Message(_)
                            | Discriminator::Choice(_)
                            | Discriminator::Confirm(_) => {}
                            _ => {
                                let msg = "'options' can only be used with 'message', 'choice', or 'confirm'";
                                return Err(de::Error::custom(msg));
                            }
                        }
                    }

                    options = options.map_or_else(
                        || Ok(Some(map.next_value::<serde_json::Value>()?)),
                        |_| Err(de::Error::custom("duplicate 'options' key found")),
                    )?;
                }
                _ => Err(de::Error::custom(format!("unexpected key: {}", key)))?,
            }
        }

        let msg = "one of 'message', 'choice', 'confirm', 'eval', 'goto', 'call', 'return' or 'exit' must be present";
        let discriminator = discriminator.ok_or(de::Error::custom(msg))?;

        let r#type = match discriminator {
            Discriminator::Message(value) => build_message(value, dem, options),
            Discriminator::Choice(value) => build_choice(value, options),
            Discriminator::Confirm(value) => build_confirm(value, options),
            Discriminator::Eval(value) => build_eval(value),
            Discriminator::Goto(value) => build_goto(value),
            Discriminator::Call(value) => build_call(value),
            Discriminator::Return(value) => build_return(value),
            Discriminator::Exit(value) => build_exit(value),
        }
        .map_err(de::Error::custom)?;

        return Ok(Line { id, r#if, r#type });
    }
}

enum Discriminator {
    Message(serde_json::Value),
    Choice(serde_json::Value),
    Confirm(serde_json::Value),
    Eval(serde_json::Value),
    Goto(serde_json::Value),
    Call(serde_json::Value),
    Return(serde_json::Value),
    Exit(serde_json::Value),
}

impl std::convert::From<(&str, serde_json::Value)> for Discriminator {
    fn from((key, value): (&str, serde_json::Value)) -> Self {
        match key {
            "message" => Discriminator::Message(value),
            "choice" => Discriminator::Choice(value),
            "confirm" => Discriminator::Confirm(value),
            "eval" => Discriminator::Eval(value),
            "goto" => Discriminator::Goto(value),
            "call" => Discriminator::Call(value),
            "return" => Discriminator::Return(value),
            "exit" => Discriminator::Exit(value),
            _ => unreachable!(),
        }
    }
}

fn build_message(
    value: serde_json::Value,
    dem: DiscriminatorEntriesMap,
    options: Option<serde_json::Value>,
) -> Result<LineType, serde_json::Error> {
    use line_type::{Message, MessageOptions, Owner, Texts};
    let message_map = dem.get("message").unwrap();
    let owner: Owner = message_map
        .get("owner")
        .cloned()
        .map(|v| serde_json::from_value(v).map_err(de::Error::custom))
        .transpose()?
        .unwrap_or_default();
    let texts: Texts = serde_json::from_value(value.clone()).map_err(de::Error::custom)?;
    let options: Option<MessageOptions> = options
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(de::Error::custom)?;
    Ok(LineType::Message(Message {
        texts,
        owner,
        options,
        is_options: false,
    }))
}

fn build_choice(
    value: serde_json::Value,
    options: Option<serde_json::Value>,
) -> Result<LineType, serde_json::Error> {
    let texts: ChoiceTexts = serde_json::from_value(value).map_err(de::Error::custom)?;
    let mut options: Option<ChoiceOptions> = options
        .clone()
        .map(|v| serde_json::from_value(v).map_err(de::Error::custom))
        .transpose()?;
    options
        .as_mut()
        .and_then(|opts| opts.message.as_mut())
        .map(|message| message.is_options = true);
    Ok(LineType::Choice(Choice { texts, options }))
}

fn build_confirm(
    value: serde_json::Value,
    options: Option<serde_json::Value>,
) -> Result<LineType, serde_json::Error> {
    use line_type::{Confirm, Texts};
    let texts: Texts = serde_json::from_value(value).map_err(de::Error::custom)?;
    let options = options
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(de::Error::custom)?;
    Ok(LineType::Confirm(Confirm { texts, options }))
}

fn build_eval(value: serde_json::Value) -> Result<LineType, serde_json::Error> {
    Ok(LineType::Eval(serde_json::from_value(value)?))
}

fn build_goto(value: serde_json::Value) -> Result<LineType, serde_json::Error> {
    Ok(LineType::Goto(serde_json::from_value(value)?))
}

fn build_call(value: serde_json::Value) -> Result<LineType, serde_json::Error> {
    Ok(LineType::Call(serde_json::from_value(value)?))
}

fn build_return(value: serde_json::Value) -> Result<LineType, serde_json::Error> {
    Ok(LineType::Return(serde_json::from_value(value)?))
}

fn build_exit(value: serde_json::Value) -> Result<LineType, serde_json::Error> {
    Ok(LineType::Exit(serde_json::from_value(value)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_message() {
        let raw_line = r#"
message:
  en: Hello world.
  ja: こんにちは、世界。
owner: 0
options:
  emotion: happy
  speed: 1.0
  listeners:
  - 1
"#
        .trim_start();
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_message_without_owner() {
        let raw_line = r#"
message:
  en: Hello world.
  ja: こんにちは、世界。
"#
        .trim_start();
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let match_result = matches!(deserialized.r#type, LineType::Message(ref message) if message.owner == Owner::default());
        assert!(match_result);
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_message_single() {
        let raw_line = "message: Hello, world.\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_choice() {
        let raw_line = r#"
choice:
- Text1
- Text2
options:
  message:
    texts: This is a choice.
    options:
      speed: 1.0
  default: bar
  timeout: 10.0
"#
        .trim_start();

        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized: String = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(raw_line, serialized);

        assert!(matches!(
            deserialized.r#type,
            LineType::Choice(line_type::Choice {
                options: Some(line_type::ChoiceOptions {
                    timeout: Some(line_type::Timeout(duration)),
                    ..
                }),
                ..
            }) if duration == std::time::Duration::from_secs_f32(10.0)
        ));
    }

    #[test]
    fn serde_confirm() {
        let raw_line = r#"
confirm:
  en: Is this ok?
  ja: これでいいですか？
options:
  response:
    yes:
      en: Yes
      ja: はい
    no:
      en: No
      ja: いいえ
"#
        .trim_start();

        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(raw_line, serialized);
    }

    #[test]
    fn serde_eval() {
        let raw_line = "eval: some_expression_here\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_goto() {
        let raw_line = "goto: 1\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);

        let raw_line = "goto: line_id_123\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_call() {
        let raw_line = "call: some_node\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_return() {
        let raw_line = "return: true\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);

        let raw_line = "return: null\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized: String = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_exit() {
        let raw_line = "exit: 0\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);

        let raw_line = "exit: 'true ? 0 : 1'\n";
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized: String = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }
}
