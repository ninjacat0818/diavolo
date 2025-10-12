use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Actors(Vec<Actor>);

impl TryFrom<serde_json::Value> for Actors {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let actors = value
            .as_array()
            .ok_or("Actors should be a JSON array of actor definitions")?
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<_, _>>()?;
        Ok(Actors(actors))
    }
}

impl FromIterator<Actor> for Actors {
    fn from_iter<I: IntoIterator<Item = Actor>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Default for Actors {
    fn default() -> Self {
        Self(vec![Actor::system()])
    }
}

impl Deref for Actors {
    type Target = Vec<Actor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Actors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Actor {
    pub(super) system: bool,
    pub(super) name: ActorName,
}

impl TryFrom<&serde_json::Value> for Actor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let map = value.as_object().ok_or("Actor should be a JSON object")?;

        let name = map
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Actor must have a 'name' field of type string")?;

        Ok(Actor {
            system: false,
            name: ActorName::from(name),
        })
    }
}

impl Actor {
    pub fn system() -> Self {
        Self {
            system: true,
            name: ActorName::system(),
        }
    }

    pub fn view_actor(&self, lang: &LanguageTag) -> ViewActor {
        let name = self
            .name
            .get(lang)
            .expect("actor name for the specified language should exist");
        ViewActor {
            system: self.system,
            name: std::borrow::Cow::Borrowed(name),
        }
    }
}

use language_tags::LanguageTag;
use std::collections::HashMap;

#[derive(PartialEq, Clone)]
pub enum ActorName {
    Monolingual(String),
    Multilingual(HashMap<LanguageTag, String>),
}

impl std::fmt::Debug for ActorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorName::Monolingual(name) => write!(f, "{}", name),
            ActorName::Multilingual(lang_names) => {
                write!(f, "{{")?;
                for (lang, name) in lang_names {
                    write!(f, "{}: {}, ", lang.as_str(), name)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl ActorName {
    pub fn get(&self, lang: &LanguageTag) -> Option<&String> {
        match self {
            ActorName::Monolingual(name) => Some(name),
            ActorName::Multilingual(lang_names) => lang_names.get(lang),
        }
    }
}

impl ActorName {
    pub fn system() -> Self {
        let system = [
            ("en", "System"),
            ("ja", "システム"),
            ("es", "Sistema"),
            ("fr", "Système"),
            ("de", "System"),
            ("zh-CN", "系统"),
            ("zh-TW", "系統"),
            ("ru", "Система"),
        ]
        .into_iter()
        .map(|(lang, name)| (lang.parse().unwrap(), name.to_string()))
        .collect::<HashMap<LanguageTag, String>>();
        ActorName::Multilingual(system)
    }
}

impl From<&str> for ActorName {
    fn from(s: &str) -> Self {
        ActorName::Monolingual(s.into())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ViewActor<'a> {
    system: bool,
    name: std::borrow::Cow<'a, str>,
}

impl ViewActor<'_> {
    pub fn is_system(&self) -> bool {
        self.system
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn into_owned(self) -> ViewActor<'static> {
        ViewActor {
            system: self.system,
            name: std::borrow::Cow::Owned(self.name.into_owned()),
        }
    }
}
