mod choice;
mod message;

pub use choice::ChoiceVisitingState;
pub use message::MessageVisitingState;

pub trait VisitingState {}

#[derive(Debug)]
pub struct OtherVisitingState {}

impl VisitingState for OtherVisitingState {}
