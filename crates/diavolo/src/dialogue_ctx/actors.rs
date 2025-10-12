use std::ops::Deref;

#[derive(Debug, Default)]
pub struct Actors(pub(super) Vec<Actor>);

impl From<Vec<Actor>> for Actors {
    fn from(actors: Vec<Actor>) -> Self {
        Self(actors)
    }
}

impl Deref for Actors {
    type Target = Vec<Actor>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Actor {
    pub(super) name: ActorName,
}

impl Actor {
    pub fn name(&self) -> &ActorName {
        &self.name
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ActorName(pub(super) String);

impl std::ops::Deref for ActorName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
