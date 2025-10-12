mod action_handler_impl;
mod view;
mod view_impl;

use super::engine::config::Config;
use super::state_machine::StateMachine;
use super::store::Store;
use dialogue::{ChoiceKey, Dialogue, Line, LineIf, LineType, Node, NodeKey, TypingSpeed};
pub use view::View;

pub struct Runner<'e, 'd> {
    store: &'e mut Store<'e>,
    dialogue: &'d Dialogue,
}

// Shortcut methods to access store components
impl Runner<'_, '_> {
    fn state(&self) -> &StateMachine {
        &self.store.data.state_machine
    }

    fn state_mut(&mut self) -> &mut StateMachine {
        &mut self.store.data.state_machine
    }

    fn config(&self) -> &Config {
        &self.store.engine.config
    }
}

impl Runner<'_, '_> {
    fn advance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_last_line() {
            tracing::debug!("Already at the last line, cannot advance further");
            self.state_mut().terminate();
            return Ok(());
        }
        let _ = self.commit_fast_forward();
        self.state_mut().advance();
        self.visit_line();
        Ok(())
    }

    fn start_fast_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.state().fast_forward.is_some() {
            return Err("Already in fast forward state".into());
        }
        self.state_mut().fast_forward = Some(std::time::Instant::now());
        Ok(())
    }

    fn release_fast_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.commit_fast_forward()?;
        self.state_mut().fast_forward = None;
        Ok(())
    }

    fn commit_fast_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use super::state_machine::visiting_states::visiting_state::{
            ChoiceVisitingState, MessageVisitingState,
        };

        let start = self
            .state()
            .fast_forward
            .ok_or("Not in fast forward state")?;

        match self.current_line_type() {
            LineType::Message(_) => {
                self.state_mut()
                    .visiting_state_mut::<MessageVisitingState>()
                    .ok_or("No MessageVisitingState found")?
                    .commit_fast_forward(start.elapsed());
                Ok(())
            }
            LineType::Choice(_) => self
                .state_mut()
                .visiting_state_mut::<ChoiceVisitingState>()
                .ok_or("No ChoiceVisitingState found")?
                .try_commit_fast_forward(start.elapsed()),
            _ => Err("Cannot commit fast forward on non-message line".into()),
        }?;

        self.state_mut().fast_forward = Some(std::time::Instant::now());
        Ok(())
    }

    fn visit_line(&mut self) {
        let line_type = self.current_line_type().clone();
        self.state_mut().visit_line(&line_type);
    }
}

impl Runner<'_, '_> {
    fn is_last_line(&self) -> bool {
        self.state().line_position + 1 == self.current_node().len()
    }

    fn current_line_type(&self) -> &LineType {
        &self.current_line().r#type
    }

    fn current_line_if(&self) -> Option<&LineIf> {
        self.current_line().r#if.as_ref()
    }

    fn current_line_speed(&self) -> Option<&TypingSpeed> {
        match self.current_line_type() {
            LineType::Message(message) => message
                .options
                .as_ref()
                .and_then(|opts| opts.speed.as_ref()),
            LineType::Choice(choice) => choice
                .options
                .as_ref()
                .and_then(|opts| opts.message.as_ref())
                .and_then(|message| message.options.as_ref())
                .and_then(|opts| opts.speed.as_ref()),
            _ => None,
        }
    }

    fn current_line(&self) -> &Line {
        &self.current_node()[self.state().line_position]
    }

    fn current_node(&self) -> &Node {
        &self.dialogue.nodes[&self.state().node_key]
    }
}

impl<'engine, 'dialogue> Runner<'engine, 'dialogue> {
    pub fn instantiate(
        store: &'engine mut Store<'engine>,
        dialogue: &'dialogue Dialogue,
    ) -> Runner<'engine, 'dialogue> {
        let mut runner = Self { store, dialogue };
        runner.init();
        runner
    }

    fn init(&mut self) {
        if self.state().is_initialidzed() {
            tracing::warn!("StateMachine is already initialized");
            return;
        }

        // if let Some(args) = args {
        //     args.register_in_rhai(&mut self.rhai.engine, &mut self.rhai.scope);
        // }
        // self.nodes = nodes.clone();

        self.state_mut().transition(Some(&NodeKey::main()), 0);
        self.visit_line();
        self.state_mut().initialize();
    }

    pub fn view(&'dialogue self) -> View<'dialogue> {
        if self.state().is_terminated() {
            return View::Terminated;
        }
        match self.current_line_type() {
            LineType::Message(message) => View::Message(self.message_view(message)),
            LineType::Choice(choice) => View::Choice(self.choice_view(choice)),
            _ => todo!("Unimplemented line type"),
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        let result = match action {
            Action::CompleteMessage => self.handle_complete_message(),
            Action::NextLine => self.handle_next_line(),
            Action::ToggleFastForward => self.handle_toggle_fast_forward(),
            Action::Skip => self.handle_skip(),
            Action::Select(ref choice_key) => self.handle_select(choice_key),
            _ => unimplemented!(),
        };

        if let Err(e) = result {
            tracing::warn!("Error handling action {:?}: {}", action, e);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CompleteMessage,
    NextLine,
    ToggleFastForward,
    Skip,
    Select(ChoiceKey),
    // Confirm(bool),
}
