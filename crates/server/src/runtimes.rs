use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Runtimes(HashMap<RuntimePath, Runtime>);

impl Deref for Runtimes {
    type Target = HashMap<RuntimePath, Runtime>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Runtimes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RuntimePath(PathBuf);

impl AsRef<str> for RuntimePath {
    fn as_ref(&self) -> &str {
        self.0.as_os_str().to_str().expect("Invalid UTF-8 in path")
    }
}

impl From<&str> for RuntimePath {
    fn from(path: &str) -> Self {
        RuntimePath(PathBuf::from(path))
    }
}

impl Deref for RuntimePath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct Runtime {
    pub engine: diavolo::Engine,
}
