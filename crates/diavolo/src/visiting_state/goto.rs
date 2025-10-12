use super::VisitingState;
use std::time::Instant;

#[derive(Debug)]
pub struct GotoVisitingState {
    visited_at: Instant,
    pub line_id_or_index: String,
}

impl GotoVisitingState {
    pub fn new(line_id_or_index: String) -> Self {
        Self {
            visited_at: Instant::now(),
            line_id_or_index,
        }
    }
}

impl VisitingState for GotoVisitingState {
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
