use super::VisitingState;
use super::message::MessageVisitingState;
use dialogue::{ChoiceKey, ChoiceTexts, LangTexts};

use std::time::Instant;

impl VisitingState for ChoiceVisitingState {}

#[derive(Debug)]
pub struct ChoiceVisitingState {
    pub started_at: Instant,
    pub evaluated_texts: ChoiceTexts,
    pub selected: Option<Selected>,
    pub message_visiting_state: Option<MessageVisitingState>,
}

impl ChoiceVisitingState {
    pub fn new(evaluated_texts: ChoiceTexts) -> Self {
        Self {
            started_at: Instant::now(),
            evaluated_texts,
            selected: None,
            message_visiting_state: None,
        }
    }

    pub fn new_with_message(
        evaluated_texts: ChoiceTexts,
        fast_forward: bool,
        evaluated_messages: LangTexts,
    ) -> Self {
        let started_at = Instant::now();
        Self {
            started_at,
            evaluated_texts,
            selected: None,
            message_visiting_state: Some(MessageVisitingState::with_sync(
                started_at,
                fast_forward,
                evaluated_messages,
            )),
        }
    }

    pub fn is_selected(&self) -> bool {
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
        self.message_visiting_state
            .as_mut()
            .map(|mvs| mvs.commit_fast_forward(duration))
            .ok_or("No message visiting state to commit fast forward".into())
    }

    pub fn try_complete_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mvs) = &mut self.message_visiting_state {
            mvs.complete();
            Ok(())
        } else {
            Err("No message visiting state to complete".into())
        }
    }

    pub fn try_skip_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mvs) = &mut self.message_visiting_state {
            if mvs.is_completed() {
                return Err("Message already completed, cannot skip".into());
            }
            mvs.skip();
            Ok(())
        } else {
            Err("No message visiting state to skip".into())
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
