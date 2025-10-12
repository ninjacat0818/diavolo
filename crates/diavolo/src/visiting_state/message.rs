use super::VisitingState;

use dialogue::LangTexts;

use std::{
    ops::AddAssign,
    time::{Duration, Instant},
};

impl VisitingState for MessageVisitingState {}

#[derive(Debug)]
pub struct MessageVisitingState {
    pub started_at: Instant,
    pub evaluated_texts: LangTexts,
    pub completed_at: Option<Instant>,
    pub skipped_at: Option<Instant>,
    pub total_fast_forward: Duration,
    pub initial_fast_forward: bool,
}

impl Default for MessageVisitingState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            evaluated_texts: LangTexts::default(),
            completed_at: Option::default(),
            skipped_at: Option::default(),
            total_fast_forward: Duration::default(),
            initial_fast_forward: bool::default(),
        }
    }
}

impl MessageVisitingState {
    pub fn new(initial_fast_forward: bool, evaluated_texts: LangTexts) -> Self {
        Self {
            initial_fast_forward,
            evaluated_texts,
            ..Default::default()
        }
    }

    pub fn with_sync(
        started_at: Instant,
        initial_fast_forward: bool,
        evaluated_texts: LangTexts,
    ) -> Self {
        Self {
            started_at,
            initial_fast_forward,
            evaluated_texts,
            ..Default::default()
        }
    }

    pub fn commit_fast_forward(&mut self, duration: Duration) {
        self.total_fast_forward.add_assign(duration);
    }

    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn complete(&mut self) {
        if self.completed_at.is_some() {
            tracing::warn!("Already in complete state");
            return;
        }
        self.completed_at.replace(Instant::now());
    }

    pub fn is_skipped(&self) -> bool {
        self.skipped_at.is_some()
    }

    pub fn skip(&mut self) {
        if self.skipped_at.is_some() {
            tracing::warn!("Already in skip state");
            return;
        }
        self.skipped_at.replace(Instant::now());
    }
}
