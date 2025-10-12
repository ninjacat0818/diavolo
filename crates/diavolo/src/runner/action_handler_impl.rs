use dialogue::ChoiceKey;

use super::Runner;
use super::super::state_machine::visiting_states::visiting_state::{ChoiceVisitingState, MessageVisitingState};
use super::view::View;
use super::view::message::MessageLifecycle;

impl Runner<'_, '_> {
    pub(super) fn handle_complete_message(
        self: &mut Self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Complete message requested");

        match self.view() {
            View::Message(message) => {
                matches!(message.lifecycle(), MessageLifecycle::Finished)
                    .then_some(())
                    .ok_or("Message is not in finished state, cannot complete message")?;

                self.state_mut()
                    .visiting_state_mut::<MessageVisitingState>()
                    .ok_or("MessageVisitingState not found")?
                    .complete();
                Ok(())
            }
            View::Choice(choice) => {
                let lifecycle = choice
                    .message_view()
                    .as_ref()
                    .ok_or("Choice has no message")?
                    .lifecycle();

                matches!(lifecycle, MessageLifecycle::Finished)
                    .then_some(())
                    .ok_or("Message is not in finished state, cannot complete message")?;

                self.state_mut()
                    .visiting_state_mut::<ChoiceVisitingState>()
                    .ok_or("ChoiceVisitingState not found")?
                    .try_complete_message()?;
                Ok(())
            }
            _ => {
                Err("Current view state is neither message nor choice, cannot complete line".into())
            }
        }
    }

    pub(super) fn handle_next_line(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Next line requested");

        match &self.view() {
            View::Terminated => {
                return Err("Dialogue has already terminated, cannot advance".into());
            }
            View::Message(message) => {
                message
                    .is_completed()
                    .then_some(())
                    .ok_or("Message is not in completed state, cannot advance")?;
            }
            View::Choice(choice) => {
                choice
                    .is_selected()
                    .then_some(())
                    .ok_or("Choice is not selected yet, cannot advance")?;
            }
            _ => todo!("Unimplemented line type"),
        }

        self.advance()
    }

    pub(super) fn handle_toggle_fast_forward(
        self: &mut Self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Toggle fast forward requested");
        if !self.state().is_fast_forward() {
            tracing::debug!("Enter fast forward");
            self.start_fast_forward()
        } else {
            tracing::debug!("Release fast forward");
            self.release_fast_forward()
        }
    }

    pub(super) fn handle_skip(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Skip requested");

        match self.view() {
            View::Message(_) => {
                self.state_mut()
                    .visiting_state_mut::<MessageVisitingState>()
                    .ok_or("MessageVisitingState not found")?
                    .skip();
                Ok(())
            }
            View::Choice(_) => {
                self.state_mut()
                    .visiting_state_mut::<ChoiceVisitingState>()
                    .ok_or("ChoiceVisitingState not found")?
                    .try_skip_message()?;
                Ok(())
            }
            _ => Err("Current view state is neither message nor choice, cannot skip".into()),
        }
    }

    pub(super) fn handle_select(
        self: &mut Self,
        choice_key: &ChoiceKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Choice selected: {:?}", choice_key);
        match self.view() {
            View::Choice(choice) => {
                choice
                    .is_available()
                    .then_some(())
                    .ok_or("Choice message is not in completed state, cannot select")?;

                if choice.is_selected() {
                    return Err("Choice already selected".into());
                } else if choice.is_expired() {
                    return Err("Choice selection period has expired".into());
                }

                self.state_mut()
                    .visiting_state_mut::<ChoiceVisitingState>()
                    .expect("ChoiceVisitingState not found")
                    .select(choice_key);

                self.handle_next_line()
            }
            _ => Err("Current view state is not choice".into()),
        }
    }
}