use super::LineState;
use super::message::MessageState;
use dialogue::{ChoiceKey, ChoiceTexts, Texts};

use std::time::Instant;

impl LineState for ChoiceState {
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
pub struct ChoiceState {
    pub visited_at: Instant,
    pub texts: ChoiceTexts,
    pub selected: Option<Selected>,
    pub message_state: Option<MessageState>,
}

impl ChoiceState {
    pub fn new(texts: ChoiceTexts) -> Self {
        Self {
            visited_at: Instant::now(),
            texts,
            selected: None,
            message_state: None,
        }
    }

    pub fn new_with_message(texts: ChoiceTexts, fast_forward: bool, messages: Texts) -> Self {
        let visited_at = Instant::now();
        Self {
            visited_at,
            texts,
            selected: None,
            message_state: Some(MessageState::with_sync(visited_at, fast_forward, messages)),
        }
    }

    pub fn _is_selected(&self) -> bool {
        self.selected.is_some()
    }

    pub fn select(&mut self, choice_key: &ChoiceKey) {
        if self.selected.is_some() {
            tracing::warn!("Choice already selected");
        }
        self.selected = Some(Selected {
            selected_at: Instant::now(),
            choice_key: choice_key.clone(),
        });
    }

    pub fn try_commit_fast_forward(
        &mut self,
        duration: std::time::Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.message_state
            .as_mut()
            .map(|message_state| message_state.commit_fast_forward(duration))
            .ok_or("No message visiting state to commit fast forward".into())
    }

    pub fn complete_message_or_panic(&mut self) {
        self.message_state
            .as_mut()
            .expect("No message visiting state to complete")
            .complete();
    }

    pub fn try_skip_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.message_state {
            None => Err("No message visiting state to skip".into()),
            Some(message_state) => {
                if message_state.is_completed() {
                    return Err("Message already completed, cannot skip".into());
                }
                message_state.skip();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Selected {
    pub selected_at: Instant,
    pub choice_key: ChoiceKey,
}

impl std::ops::Deref for Selected {
    type Target = ChoiceKey;

    fn deref(&self) -> &Self::Target {
        &self.choice_key
    }
}
