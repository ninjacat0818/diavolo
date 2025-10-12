pub mod call;
pub mod choice;
pub mod eval;
pub mod goto;
pub mod message;
pub mod r#return;

pub use call::CallVisitingState;
pub use choice::ChoiceVisitingState;
pub use eval::EvalVisitingState;
pub use goto::GotoVisitingState;
pub use message::MessageVisitingState;
pub use r#return::ReturnVisitingState;

#[allow(unused)]
pub trait VisitingState: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn visited_at(&self) -> std::time::Instant;
}
