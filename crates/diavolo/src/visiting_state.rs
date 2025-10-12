pub mod choice;
pub mod message;

pub use choice::ChoiceVisitingState;
pub use message::MessageVisitingState;

#[allow(unused)]
pub trait VisitingState {}

#[derive(Debug)]
pub struct OtherVisitingState {}

impl VisitingState for OtherVisitingState {}
