use super::dialogue_ctx::DialogueCtx;
use super::state_machine::StateMachine;

#[derive(Debug, Default)]
pub struct Data {
    pub(crate) dialogue_ctx: DialogueCtx,
    pub(crate) state_machine: StateMachine,
}

impl Data {
    pub fn with_ctx(dialogue_ctx: DialogueCtx) -> Self {
        Self {
            dialogue_ctx,
            ..Default::default()
        }
    }
}