use super::dialogue_ctx::DialogueCtx;
use super::runner::EvaluatedLine;
use super::state_machine::StateMachine;
use super::view::View;
use super::line_state::*;
use super::visiting_states::{Lines, VisitingStates};

use dialogue::{NodeKey, Nodes};

#[derive(Debug, Default)]
pub struct Data {
    pub dialogue_ctx: DialogueCtx,
    pub(crate) state_machine: StateMachine,
    pub(crate) visiting_states: VisitingStates,
    pub(crate) exit_code: Option<u8>,
}

impl Data {
    pub fn with_ctx(dialogue_ctx: DialogueCtx) -> Self {
        Self {
            dialogue_ctx,
            ..Default::default()
        }
    }

    pub(crate) fn goto(&mut self, nodes: &Nodes) {
        let line_id_or_index = self
            .visiting_state_or_panic::<GotoState>()
            .line_id_or_index
            .as_str();

        let line_position = line_id_or_index
            .parse::<usize>()
            .map(|idx| idx.into())
            .unwrap_or_else(|_| {
                let node: &dialogue::Node = nodes
                    .get(self.state_machine.current_node_key())
                    .expect("Current node should exist");
                node.iter()
                    .enumerate()
                    .find(|(_, line)| {
                        line.id
                            .as_ref()
                            .map(|id| **id == *line_id_or_index)
                            .unwrap_or_default()
                    })
                    .map(|(idx, _)| idx)
                    .expect(format!("Line ID {:?} does not exist", line_id_or_index).as_str())
                    .into()
            });

        self.state_machine.goto(line_position);
    }

    pub(crate) fn call(&mut self, nodes: &Nodes) {
        let node_key = self
            .state_machine
            .call_stack
            .is_empty()
            .then_some(NodeKey::main())
            .unwrap_or_else(|| {
                self.visiting_state_or_panic::<CallState>()
                    .node_key
                    .clone()
            });
        let node = nodes
            .get(&node_key)
            .expect(format!("Node {:?} does not exist", node_key).as_str());
        self.state_machine.call(node_key.clone());
        self.visiting_states.ensure_node(node_key, node);
    }

    pub(crate) fn ret(&mut self) {
        let value = self
            .visiting_state::<ReturnState>()
            .map(|r| &r.value)
            .cloned()
            .unwrap_or_default();
        self.state_machine.call_stack.pop();
        if !self.state_machine.call_stack.is_empty() {
            self.visiting_state_mut_or_panic::<CallState>()
                .ret(value);
        }
    }

    pub(crate) fn exit(&mut self, code: u8) {
        self.exit_code = Some(code);
        self.state_machine.call_stack.clear();
    }

    pub(crate) fn visit_line(&mut self, evaluated_line: EvaluatedLine) {
        self.visiting_states.visit_line(
            self.state_machine.location(),
            self.state_machine.is_fast_forward(),
            evaluated_line,
        )
    }

    pub(crate) fn lines_or_panic(&self) -> &Lines {
        self.visiting_states
            .lines(self.state_machine.current_node_key())
            .expect("Lines should exist for the current node key")
    }

    pub(crate) fn complete_message_or_panic(&mut self, view: &View) {
        use super::line_state::{ChoiceState, MessageState};
        match view {
            View::Message(_) => {
                self.visiting_state_mut_or_panic::<MessageState>()
                    .complete();
            }
            View::Confirm(_) => {
                self.visiting_state_mut_or_panic::<ConfirmState>()
                    .complete();
            }
            View::Choice(_) => {
                self.visiting_state_mut_or_panic::<ChoiceState>()
                    .complete_message_or_panic();
            }
            _ => panic!("No message to complete in the current view"),
        }
    }

    pub(crate) fn visiting_state<T: LineState>(&self) -> Option<&T> {
        self.visiting_states
            .visiting_state::<T>(self.state_machine.location())
    }

    pub(crate) fn visiting_state_or_panic<T: LineState>(&self) -> &T {
        self.visiting_state::<T>().unwrap()
    }

    pub(crate) fn visiting_state_mut<T: LineState>(&mut self) -> Option<&mut T> {
        self.visiting_states
            .visiting_state_mut::<T>(self.state_machine.location())
    }

    pub(crate) fn visiting_state_mut_or_panic<T: LineState>(&mut self) -> &mut T {
        self.visiting_state_mut::<T>().unwrap()
    }
}
