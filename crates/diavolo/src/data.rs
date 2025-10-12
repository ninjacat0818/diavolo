use super::dialogue_ctx::DialogueCtx;
use super::state_machine::StateMachine;
use super::visiting_state::{ChoiceVisitingState, MessageVisitingState};
use super::visiting_states::{Lines, VisitingStates};

#[derive(Debug, Default)]
pub struct Data {
    pub dialogue_ctx: DialogueCtx,
    pub(crate) state_machine: StateMachine,
    pub(crate) visiting_states: VisitingStates,
}

impl Data {
    pub fn with_ctx(dialogue_ctx: DialogueCtx) -> Self {
        Self {
            dialogue_ctx,
            ..Default::default()
        }
    }

    pub(crate) fn visit_line(
        &mut self,
        evaluated_texts: Option<dialogue::LangTexts>,
        evaluated_choice_texts: Option<dialogue::ChoiceTexts>,
    ) -> boa_engine::JsResult<()> {
        let sm = &self.state_machine;
        self.visiting_states.visit_line(
            &sm.node_key,
            sm.line_position,
            sm.is_fast_forward(),
            evaluated_texts,
            evaluated_choice_texts,
        )
    }

    pub(crate) fn lines(&self) -> &Lines {
        self.visiting_states
            .lines(&self.state_machine.node_key)
            .expect("Lines should exist for the current node key")
    }

    pub(crate) fn message_visiting_state(&self) -> &MessageVisitingState {
        self.visiting_states
            .message_visiting_state(
                &self.state_machine.node_key,
                self.state_machine.line_position,
            )
            .expect("MessageVisitingState should exist for the current line")
    }

    pub(crate) fn message_visiting_state_mut(&mut self) -> &mut MessageVisitingState {
        self.visiting_states
            .message_visiting_state_mut(
                &self.state_machine.node_key,
                self.state_machine.line_position,
            )
            .expect("MessageVisitingState should exist for the current line")
    }

    pub(crate) fn choice_visiting_state(&self) -> &ChoiceVisitingState {
        self.visiting_states
            .choice_visiting_state(
                &self.state_machine.node_key,
                self.state_machine.line_position,
            )
            .expect("ChoiceVisitingState should exist for the current line")
    }

    pub(crate) fn choice_visiting_state_mut(&mut self) -> &mut ChoiceVisitingState {
        self.visiting_states
            .choice_visiting_state_mut(
                &self.state_machine.node_key,
                self.state_machine.line_position,
            )
            .expect("ChoiceVisitingState should exist for the current line")
    }
}
