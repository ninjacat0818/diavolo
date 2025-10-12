use isolang::Language;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(transparent)]
pub struct LangTexts(BTreeMap<Language, Text>);

impl Default for LangTexts {
    fn default() -> Self {
        let mut map = BTreeMap::new();
        map.insert(Language::from_639_1("en").unwrap(), Text::default());
        LangTexts(map)
    }
}

impl Deref for LangTexts {
    type Target = BTreeMap<Language, Text>;

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

        for (language, text) in &self.0 {
            let lang_code = language.to_639_1().unwrap_or("en");
            map.serialize_entry(lang_code, text)?;
        }

        map.end()
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Text(String);

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text(s)
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Text {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
