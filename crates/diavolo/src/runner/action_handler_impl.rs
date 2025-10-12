use super::super::view::View;
use super::super::view::message::MessageLifecycle;
use super::Runner;

use dialogue::ChoiceKey;

impl Runner<'_, '_> {
    pub(super) fn handle_complete_message(
        self: &mut Self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Complete message requested");
        let mut data = self.store.data.lock().unwrap();

        match &self.view {
            View::Message(message) => {
                matches!(message.lifecycle(), MessageLifecycle::Finished)
                    .then_some(())
                    .ok_or("Message is not in finished state, cannot complete message")?;
                data.message_visiting_state_mut().complete();
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

                data.choice_visiting_state_mut().try_complete_message()?;
                Ok(())
            }
            _ => {
                Err("Current view state is neither message nor choice, cannot complete line".into())
            }
        }
    }

    pub(super) fn handle_next_line(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Next line requested");

        match &self.view {
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
        let mut data = self.store.data.lock().unwrap();
        let state_machine = &mut data.state_machine;

        if !state_machine.is_fast_forward() {
            tracing::debug!("Enter fast forward");
            state_machine.enter_fast_forward();
        } else {
            drop(data);
            tracing::debug!("Release fast forward");
            self.commit_fast_forward()?;
            self.store
                .data
                .lock()
                .unwrap()
                .state_machine
                .leave_fast_forward()
        }

        Ok(())
    }

    pub(super) fn handle_skip(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Skip requested");
        let mut data = self.store.data.lock().unwrap();
        match &self.view {
            View::Message(_) => {
                data.message_visiting_state_mut().skip();
                Ok(())
            }
            View::Choice(_) => data.choice_visiting_state_mut().try_skip_message(),
            _ => Err("Current view state is neither message nor choice, cannot skip".into()),
        }
    }

    pub(super) fn handle_select(
        self: &mut Self,
        choice_key: &ChoiceKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Choice selected: {:?}", choice_key);

        match &self.view {
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

                let mut data = self.store.data.lock().unwrap();
                data.choice_visiting_state_mut().select(choice_key);
                drop(data);

                self.handle_next_line()
            }
            _ => Err("Current view state is not choice".into()),
        }
    }
}
