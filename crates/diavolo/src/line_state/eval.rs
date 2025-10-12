use super::LineState;
use std::time::Instant;

#[derive(Debug)]
pub struct EvalState {
    visited_at: Instant,
    pub value: boa_engine::JsValue,
}

impl EvalState {
    pub fn new(value: boa_engine::JsValue) -> Self {
        Self {
            visited_at: Instant::now(),
            value,
        }
    }
}

impl LineState for EvalState {
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
