use super::VisitingState;
use std::{
    ops::AddAssign,
    time::{Duration, Instant},
};

impl VisitingState for MessageVisitingState {}

#[derive(Debug)]
pub struct MessageVisitingState {
    started_at: Instant,
    completed_at: Option<Instant>,
    skipped_at: Option<Instant>,
    total_fast_forward: Duration,
    initial_fast_forward: bool,
}

impl Default for MessageVisitingState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            completed_at: None,
            skipped_at: None,
            total_fast_forward: Duration::default(),
            initial_fast_forward: false,
        }
    }
}

impl MessageVisitingState {
    pub fn new(initial_fast_forward: bool) -> Self {
        Self {
            initial_fast_forward,
            ..Default::default()
        }
    }

    pub fn with_sync(started_at: Instant, initial_fast_forward: bool) -> Self {
        Self {
            started_at,
            initial_fast_forward,
            ..Default::default()
        }
    }

    pub fn started_at(&self) -> &Instant {
        &self.started_at
    }

    pub fn is_fast_forwarded(&self) -> bool {
        !self.total_fast_forward.is_zero()
    }

    pub fn is_initial_fast_forward(&self) -> bool {
        self.initial_fast_forward
    }

    pub fn total_fast_forward(&self) -> &Duration {
        &self.total_fast_forward
    }

    pub fn commit_fast_forward(&mut self, duration: Duration) {
        self.total_fast_forward.add_assign(duration);
    }

    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn completed_at(&self) -> &Option<Instant> {
        &self.completed_at
    }

    pub fn complete(&mut self) {
        if self.completed_at.is_some() {
            tracing::warn!("Already in complete state");
        }
        self.completed_at = Some(Instant::now());
    }

    pub fn is_skipped(&self) -> bool {
        self.skipped_at.is_some()
    }

    pub fn skipped_at(&self) -> &Option<Instant> {
        &self.skipped_at
    }

    pub fn skip(&mut self) {
        if self.skipped_at.is_some() {
            tracing::warn!("Already in skip state");
        }
        self.skipped_at = Some(Instant::now());
    }
}
