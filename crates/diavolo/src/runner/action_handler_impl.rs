use super::super::view::View;
use super::super::line_state::{
    ChoiceState, ConfirmState, MessageState,
};
use super::Runner;

use dialogue::ChoiceKey;

impl Runner<'_, '_> {
    pub(super) fn handle_advance(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Advance requested");

        match View::new(
            &self.store.engine,
            &mut self.store.data.lock().unwrap(),
            &self.dialogue.nodes,
        ) {
            View::Terminated(code) => {
                return Err(format!(
                    "Dialogue has already terminated with exit code {}, cannot advance",
                    code
                )
                .into());
            }
            View::Message(message) => {
                message
                    .is_completed()
                    .then_some(())
                    .ok_or("Message is not in completed state, cannot advance")?;
            }
            View::Confirm(confirm) => {
                confirm
                    .is_confirmed()
                    .then_some(())
                    .ok_or("Confirm is not confirmed yet, cannot advance")?;
            }
            View::Choice(choice) => {
                choice
                    .is_selected()
                    .then_some(())
                    .ok_or("Choice is not selected yet, cannot advance")?;
            }
            _ => todo!("Unimplemented line type"),
        }

        Ok(self.advance()?)
    }

    pub(super) fn handle_toggle_fast_forward(
        self: &mut Self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Toggle fast forward requested");
        let mut data = self.store.data.lock().unwrap();
        let state_machine = &mut data.state_machine;

        if !state_machine.is_fast_forward() {
            tracing::debug!("Enter fast forward");
            state_machine
                .fast_forward
                .replace(std::time::Instant::now());
        } else {
            tracing::debug!("Release fast forward");
            Self::try_commit_fast_forward(&mut data, &self.dialogue.nodes, false)?;
        }

        Ok(())
    }

    pub(super) fn handle_skip(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Skip requested");
        let mut data = self.store.data.lock().unwrap();
        let view = View::new(&self.store.engine, &mut data, &self.dialogue.nodes);

        match view {
            View::Message(_) => {
                data.visiting_state_mut_or_panic::<MessageState>()
                    .skip();
                Ok(())
            }
            View::Confirm(_) => {
                data.visiting_state_mut_or_panic::<ConfirmState>()
                    .message_state
                    .skip();
                Ok(())
            }
            View::Choice(_) => data
                .visiting_state_mut_or_panic::<ChoiceState>()
                .try_skip_message(),
            _ => Err("Current view state is neither message nor choice, cannot skip".into()),
        }
    }

    pub(super) fn handle_confirm(
        self: &mut Self,
        approved: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Confirm response: {:?}", approved);
        let mut data = self.store.data.lock().unwrap();
        let view = View::new(&self.store.engine, &mut data, &self.dialogue.nodes);

        match view {
            View::Confirm(confirm) => {
                if !confirm.is_available() {
                    return Err("Confirm already responded or message is not completed yet".into());
                }

                data.visiting_state_mut_or_panic::<ConfirmState>()
                    .confirmed
                    .replace(approved);
                drop(data);

                self.handle_advance()
            }
            _ => Err("Current view state is not confirm".into()),
        }
    }

    pub(super) fn handle_select(
        self: &mut Self,
        choice_key: &ChoiceKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!("Choice selected: {:?}", choice_key);
        let mut data = self.store.data.lock().unwrap();
        let view = View::new(&self.store.engine, &mut data, &self.dialogue.nodes);

        match view {
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

                data.visiting_state_mut_or_panic::<ChoiceState>()
                    .select(choice_key);
                drop(data);

                self.handle_advance()
            }
            _ => Err("Current view state is not choice".into()),
        }
    }
}
