use super::LineState;
use super::message::MessageState;
use dialogue::{ConfirmResponse, Texts};

use std::time::Instant;

impl LineState for ConfirmState {
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
pub struct ConfirmState {
    pub visited_at: Instant,
    pub response_texts: Option<ConfirmResponse>,
    pub confirmed: Option<bool>,
    pub message_state: MessageState,
}

impl std::ops::Deref for ConfirmState {
    type Target = MessageState;

    fn deref(&self) -> &Self::Target {
        &self.message_state
    }
}

impl std::ops::DerefMut for ConfirmState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.message_state
    }
}

impl ConfirmState {
    pub fn new(
        initial_fast_forward: bool,
        texts: Texts,
        response_texts: Option<ConfirmResponse>,
    ) -> Self {
        let visited_at = Instant::now();
        Self {
            visited_at,
            response_texts,
            confirmed: None,
            message_state: MessageState::with_sync(
                visited_at,
                initial_fast_forward,
                texts,
            ),
        }
    }
}
