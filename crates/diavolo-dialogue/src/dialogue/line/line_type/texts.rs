mod lang_texts;
mod text;

pub use lang_texts::LangTexts;
pub use text::Text;

use language_tags::LanguageTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Texts {
    Monolingual(Text),
    Multilingual(LangTexts),
}

impl Texts {
    pub fn get(&self, lang: &LanguageTag) -> Option<&Text> {
        match self {
            Texts::Monolingual(text) => Some(text),
            Texts::Multilingual(lang_texts) => lang_texts.get(lang),
        }
    }
}

impl Default for Texts {
    fn default() -> Self {
        Texts::Multilingual(LangTexts::default())
    }
}
