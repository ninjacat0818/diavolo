use super::text::Text;

use language_tags::LanguageTag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(transparent)]
pub struct LangTexts(HashMap<LanguageTag, Text>);

impl FromIterator<(LanguageTag, Text)> for LangTexts {
    fn from_iter<T: IntoIterator<Item = (LanguageTag, Text)>>(iter: T) -> Self {
        let mut map = HashMap::new();
        for (language, text) in iter {
            map.insert(language, text);
        }
        LangTexts(map)
    }
}

impl Default for LangTexts {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(LanguageTag::parse("en").unwrap(), Text::default());
        LangTexts(map)
    }
}

impl Deref for LangTexts {
    type Target = HashMap<LanguageTag, Text>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for LangTexts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        let mut entries: Vec<_> = self
            .iter()
            .map(|(lang_tag, text)| {
                let lang_tag = lang_tag
                    .canonicalize()
                    .map_err(serde::ser::Error::custom)?
                    .to_string();
                Ok::<_, S::Error>((lang_tag, text))
            })
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (lang_tag, text) in &entries {
            map.serialize_entry(lang_tag, text)?;
        }

        map.end()
    }
}
