use super::data::Data;
use super::engine::Engine;

use std::sync::{Arc, Mutex};

pub struct Store<'e> {
    pub engine: &'e Engine,
    pub data: Arc<Mutex<Data>>,
}

impl<'e> Store<'e> {
    pub fn new(engine: &'e Engine, data: Data) -> Store<'e> {
        Store {
            engine,
            data: Arc::new(Mutex::new(data)),
        }
    }
}
