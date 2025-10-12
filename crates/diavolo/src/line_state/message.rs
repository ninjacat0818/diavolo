use super::LineState;

use dialogue::Texts;

use std::{
    ops::AddAssign,
    time::{Duration, Instant},
};

impl LineState for MessageState {
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
pub struct MessageState {
    pub visited_at: Instant,
    pub texts: Texts,
    pub completed_at: Option<Instant>,
    pub skipped_at: Option<Instant>,
    pub total_fast_forward: Duration,
    pub initial_fast_forward: bool,
}

impl Default for MessageState {
    fn default() -> Self {
        Self {
            visited_at: Instant::now(),
            texts: Texts::default(),
            completed_at: Option::default(),
            skipped_at: Option::default(),
            total_fast_forward: Duration::default(),
            initial_fast_forward: bool::default(),
        }
    }
}

impl MessageState {
    pub fn new(initial_fast_forward: bool, texts: Texts) -> Self {
        Self {
            initial_fast_forward,
            texts,
            ..Default::default()
        }
    }

    pub fn with_sync(visited_at: Instant, initial_fast_forward: bool, texts: Texts) -> Self {
        Self {
            visited_at,
            initial_fast_forward,
            texts,
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
        } else if self.completed_at.is_some() {
            tracing::warn!("Already in complete state, skipping has no effect");
            return;
        }
        let now = Instant::now();
        self.skipped_at.replace(now);
        self.completed_at.replace(now);
    }
}
