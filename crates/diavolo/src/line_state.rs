pub mod call;
pub mod choice;
pub mod confirm;
pub mod eval;
pub mod goto;
pub mod message;
pub mod r#return;

pub use call::CallState;
pub use choice::ChoiceState;
pub use confirm::ConfirmState;
pub use eval::EvalState;
pub use goto::GotoState;
pub use message::MessageState;
pub use r#return::ReturnState;

#[allow(unused)]
pub trait LineState: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn visited_at(&self) -> std::time::Instant;
}
