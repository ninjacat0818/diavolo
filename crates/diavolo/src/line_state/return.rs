use super::LineState;

use boa_engine::JsValue;
use std::time::Instant;

impl LineState for ReturnState {
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

#[derive(Debug)]
pub struct ReturnState {
    visited_at: Instant,
    pub value: JsValue,
}

impl ReturnState {
    pub fn new(value: JsValue) -> Self {
        Self {
            visited_at: Instant::now(),
            value,
        }
    }
}
