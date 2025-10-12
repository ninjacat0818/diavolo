use super::data::Data;
use super::engine::Engine;

pub struct Store<'e> {
    pub engine: &'e Engine,
    pub data: Data,
}

impl<'e> Store<'e> {
    pub fn new(engine: &'e Engine, data: Data) -> Store<'e> {
        Store { engine, data }
    }
}
