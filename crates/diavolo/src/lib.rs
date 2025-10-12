mod data;
mod dialogue_ctx;
mod engine;
mod runner;
mod state_machine;
mod store;

pub extern crate dialogue;

pub use data::Data;
pub use dialogue::Dialogue;
pub use engine::Engine;
pub use runner::{Action, Runner, View};
pub use store::Store;

mod tests {
    #[test]
    fn test() {
        use super::*;

        let engine = Engine::default();
        let mut store = Store::new(&engine, Data::with_ctx(dialogue_ctx::DialogueCtx::new()));
        let dialogue = Dialogue::default();
        let runner = Runner::instantiate(&mut store, &dialogue);
    }
}
