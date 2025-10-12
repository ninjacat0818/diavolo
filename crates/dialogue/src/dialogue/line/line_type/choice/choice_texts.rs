use super::super::texts::Texts;

use indexmap::IndexMap;
use serde::de;
use serde::de::value::{MapAccessDeserializer, SeqAccessDeserializer};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone)]
pub struct ChoiceTexts(IndexMap<ChoiceKey, Texts>);

impl ChoiceTexts {
    fn is_seq(&self) -> bool {
        self.keys()
            .enumerate()
            .all(|(i, key)| key.0 == i.to_string())
    }
}

impl FromIterator<(ChoiceKey, Texts)> for ChoiceTexts {
    fn from_iter<T: IntoIterator<Item = (ChoiceKey, Texts)>>(iter: T) -> Self {
        let inner = IndexMap::from_iter(iter);
        ChoiceTexts(inner)
    }
}

impl Deref for ChoiceTexts {
    type Target = IndexMap<ChoiceKey, Texts>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for ChoiceTexts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        if self.is_seq() {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for lang_texts in self.values() {
                seq.serialize_element(lang_texts)?;
            }
            seq.end()
        } else {
            let mut map = serializer.serialize_map(Some(self.len()))?;
            for (choice_key, lang_texts) in &self.0 {
                map.serialize_entry(&choice_key.0, lang_texts)?;
            }
            map.end()
        }
    }
}

impl<'de> Deserialize<'de> for ChoiceTexts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ChoiceTexts;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map or a sequence of choice texts")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let inner = Vec::<Texts>::deserialize(SeqAccessDeserializer::new(seq))?
                    .into_iter()
                    .enumerate()
                    .map(|(i, lang_texts)| (ChoiceKey(i.to_string()), lang_texts))
                    .collect::<IndexMap<ChoiceKey, Texts>>();
                if inner.is_empty() {
                    return Err(de::Error::custom("Choice texts sequence cannot be empty"));
                }
                Ok(ChoiceTexts(inner))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let inner: IndexMap<ChoiceKey, Texts> =
                    Deserialize::deserialize(MapAccessDeserializer::new(map))?;
                if inner.is_empty() {
                    return Err(de::Error::custom("Choice texts map cannot be empty"));
                }
                Ok(ChoiceTexts(inner))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub struct ChoiceKey(String);

impl ChoiceKey {
    pub fn new(key: impl Into<String>) -> Self {
        ChoiceKey(key.into())
    }
}

impl Deref for ChoiceKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn serde_choice_texts_map() {
        let raw = r#"
foo:
  en: Foo
bar:
  en: Bar
"#
        .trim_start();
        let deserialized: ChoiceTexts = serde_yaml::from_str(raw).unwrap();
        let serialized = serde_yaml::to_string(&deserialized).unwrap();
        assert_eq!(raw, serialized);
    }

    #[test]
    fn serde_choice_texts_seq() {
        let raw = r#"
- en: Foo
- en: Bar
"#
        .trim_start();
        let choice_texts: ChoiceTexts = serde_yaml::from_str(raw).unwrap();
        let serialized = serde_yaml::to_string(&choice_texts).unwrap();
        assert_eq!(raw, serialized);
    }

    #[test]
    fn deserialize_choice_texts_empty_map_error() {
        let raw = r#"{}"#;
        let result: Result<ChoiceTexts, _> = serde_yaml::from_str(raw);
        assert!(result.is_err());
    }
}
