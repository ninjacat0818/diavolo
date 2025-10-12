use super::VisitingState;
use std::time::Instant;

#[derive(Debug)]
pub struct EvalVisitingState {
    visited_at: Instant,
    pub value: boa_engine::JsValue,
}

impl EvalVisitingState {
    pub fn new(value: boa_engine::JsValue) -> Self {
        Self {
            visited_at: Instant::now(),
            value,
        }
    }
}

impl VisitingState for EvalVisitingState {
    fn visited_at(&self) -> Instant {
        self.visited_at
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
