use super::line_type::{self, LineType};
use serde::{Deserialize, Serialize, de};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct LineIf(String);

impl AsRef<str> for LineIf {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for LineIf {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Line {
    pub r#if: Option<LineIf>,
    pub r#type: LineType,
}

impl Serialize for Line {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        if let Some(r#if) = &self.r#if {
            map.serialize_entry("if", r#if)?;
        }

        match &self.r#type {
            LineType::Message(message) => {
                map.serialize_entry("message", &message.texts)?;
                map.serialize_entry("owner", &message.owner)?;
                if let Some(listeners) = &message.listeners {
                    map.serialize_entry("listeners", listeners)?;
                }
                if let Some(options) = &message.options {
                    map.serialize_entry("options", options)?;
                }
            }
            LineType::Choice(choice) => {
                map.serialize_entry("choice", &choice.texts)?;
                if let Some(options) = &choice.options {
                    map.serialize_entry("options", options)?;
                }
            }
            LineType::Confirm(confirm) => {
                map.serialize_entry("confirm", &confirm.texts)?;
                if let Some(options) = &confirm.options {
                    map.serialize_entry("options", options)?;
                }
            }
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Line {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MapVisitor;

        use std::fmt;
        impl<'de> serde::de::Visitor<'de> for MapVisitor {
            type Value = Line;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map representing a Line")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                enum Discriminator {
                    Message(serde_json::Value),
                    Choice(serde_json::Value),
                    Confirm(serde_json::Value),
                }

                impl std::convert::From<(&str, serde_json::Value)> for Discriminator {
                    fn from((key, value): (&str, serde_json::Value)) -> Self {
                        match key {
                            "message" => Discriminator::Message(value),
                            "choice" => Discriminator::Choice(value),
                            "confirm" => Discriminator::Confirm(value),
                            _ => unreachable!(),
                        }
                    }
                }

                let mut discriminator: Option<Discriminator> = None;
                let mut r#if: Option<LineIf> = None;
                let mut options: Option<serde_json::Value> = None;

                use std::collections::HashMap;
                let mut discriminator_entries_map: HashMap<
                    &str,
                    HashMap<String, serde_json::Value>,
                > = HashMap::from([
                    ("message", HashMap::new()),
                    ("choice", HashMap::new()),
                    ("confirm", HashMap::new()),
                ]);

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_ref() {
                        k @ ("message" | "choice" | "confirm") => {
                            if discriminator.is_some() {
                                return Err(de::Error::custom(format!(
                                    "mutually exclusive keys present: {k}",
                                )));
                            }

                            let conflicting_entry = discriminator_entries_map
                                .iter()
                                .filter(|(discriminator_name, _)| **discriminator_name != k)
                                .find_map(|(name, entries)| {
                                    (!entries.is_empty())
                                        .then(|| (*name, entries.keys().next().unwrap().as_str()))
                                });

                            if let Some((discriminator_name, conflicting_key)) = conflicting_entry {
                                return Err(de::Error::custom(format!(
                                    "'{conflicting_key}' can only be used with: {discriminator_name}"
                                )));
                            }

                            let value = map.next_value::<serde_json::Value>()?;
                            discriminator = Some(Discriminator::from((k, value)));
                        }
                        k @ ("owner" | "listeners") => {
                            if discriminator.is_some()
                                && !matches!(discriminator, Some(Discriminator::Message(_)))
                            {
                                return Err(de::Error::custom(format!(
                                    "'{k}' can only be used with 'message'"
                                )));
                            }

                            let entries = discriminator_entries_map.get_mut("message").unwrap();
                            if entries.contains_key(k) {
                                return Err(de::Error::custom(format!("duplicate key found: {k}")));
                            }
                            entries.insert(k.to_owned(), map.next_value::<serde_json::Value>()?);
                        }
                        "if" => {
                            r#if = r#if.map_or_else(
                                || Ok(Some(map.next_value::<LineIf>()?)),
                                |_| Err(de::Error::custom("duplicate 'if' key found")),
                            )?;
                        }
                        "options" => {
                            options = options.map_or_else(
                                || Ok(Some(map.next_value::<serde_json::Value>()?)),
                                |_| Err(de::Error::custom("duplicate 'options' key found")),
                            )?;
                        }
                        _ => Err(de::Error::custom(format!("unexpected key: {}", key)))?,
                    }
                }

                match discriminator.ok_or(de::Error::custom(
                    "one of 'message', 'choice', or 'confirm' must be present",
                ))? {
                    Discriminator::Message(value) => {
                        use line_type::{LangTexts, Listeners, Message, MessageOptions, Owner};

                        let message_map = discriminator_entries_map.get_mut("message").unwrap();

                        let owner: Owner = message_map
                            .remove("owner")
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(de::Error::custom)?
                            .ok_or(de::Error::custom(
                                "'owner' must be present when 'message' is used",
                            ))?;

                        let listeners: Option<Listeners> = message_map
                            .remove("listeners")
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(de::Error::custom)?;

                        let texts: LangTexts =
                            serde_json::from_value(value.clone()).map_err(de::Error::custom)?;

                        let options: Option<MessageOptions> = options
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(de::Error::custom)?;

                        Ok(Line {
                            r#if,
                            r#type: LineType::Message(Message {
                                texts,
                                owner,
                                listeners,
                                options,
                            }),
                        })
                    }
                    Discriminator::Choice(value) => {
                        use line_type::{Choice, ChoiceOptions, ChoiceTexts};

                        let _choice_map = discriminator_entries_map.get_mut("choice").unwrap();

                        let texts: ChoiceTexts =
                            serde_json::from_value(value).map_err(de::Error::custom)?;

                        let options: Option<ChoiceOptions> = options
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(de::Error::custom)?;

                        Ok(Line {
                            r#if,
                            r#type: LineType::Choice(Choice { texts, options }),
                        })
                    }
                    Discriminator::Confirm(value) => {
                        use line_type::{Confirm, LangTexts};

                        let texts: LangTexts =
                            serde_json::from_value(value.clone()).map_err(de::Error::custom)?;

                        let options = options
                            .map(serde_json::from_value)
                            .transpose()
                            .map_err(de::Error::custom)?;

                        Ok(Line {
                            r#if,
                            r#type: LineType::Confirm(Confirm { texts, options }),
                        })
                    }
                }
            }
        }

        deserializer.deserialize_map(MapVisitor)
    }
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
listeners:
- 1
options:
  emotion: happy
  speed: 1.0
"#
        .trim_start();
        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(serialized, raw_line);
    }

    #[test]
    fn serde_choice() {
        let raw_line = r#"
choice:
  foo:
    en: Foo Message
  bar:
    en: Bar Message
options:
  message:
    texts:
      en: This is a choice.
    owner: 0
    options:
      speed: 1.0
  default: bar
  timeout: 10.0
"#
        .trim_start();

        let deserialized: Line = serde_yaml::from_str(raw_line).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
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
}
