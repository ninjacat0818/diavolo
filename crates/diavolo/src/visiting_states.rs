use super::line_state::choice::Selected;
use super::line_state::*;
use super::runner::EvaluatedLine;
use dialogue::{LineId, LineType, Location, NodeKey};

use boa_engine::JsValue;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct VisitingStates(HashMap<NodeKey, Node>);

impl VisitingStates {
    pub fn ensure_node(&mut self, node_key: NodeKey, node: &dialogue::Node) {
        if self.contains_key(&node_key) {
            tracing::debug!("Node already initialized for node_key: {}", node_key);
            return;
        }

        tracing::debug!("Initializing node for node_key: {}", node_key);
        let vs_node = self.entry(node_key).or_default();

        for line in node.iter() {
            let line_id = line.id.clone().unwrap_or_else(|| LineId::unique());
            vs_node.insert(line_id, VisitingCounting::from(&line.r#type));
        }
    }

    pub fn visit_line(
        &mut self,
        location: &Location,
        initial_fast_forward: bool,
        evaluated_line: EvaluatedLine,
    ) {
        let visiting_counting = self
            .get_mut(&location.node_key)
            .expect("Node must be initialized before visiting lines")
            .get_index_mut(*location.line_position)
            .map(|(_, vc)| vc)
            .expect("Line must be initialized before visiting");

        match visiting_counting {
            VisitingCounting::Message(states) => {
                states.push(MessageState::new(
                    initial_fast_forward,
                    evaluated_line.into_message_or_panic(),
                ));
            }
            VisitingCounting::Confirm(states) => {
                let (texts, response_texts) = evaluated_line.into_confirm_or_panic();
                states.push(ConfirmState::new(
                    initial_fast_forward,
                    texts,
                    response_texts,
                ));
            }
            VisitingCounting::Choice(states) => {
                let (choice_texts, texts) = evaluated_line.into_choice_or_panic();
                let state = match texts {
                    Some(texts) => {
                        ChoiceState::new_with_message(choice_texts, initial_fast_forward, texts)
                    }
                    None => ChoiceState::new(choice_texts),
                };
                states.push(state);
            }
            VisitingCounting::Eval(states) => {
                states.push(EvalState::new(evaluated_line.into_eval_or_panic()));
            }
            VisitingCounting::Goto(states) => {
                states.push(GotoState::new(evaluated_line.into_goto_or_panic()));
            }
            VisitingCounting::Call(states) => {
                states.push(CallState::new(evaluated_line.into_call_or_panic()));
            }
            VisitingCounting::Return(states) => {
                states.push(ReturnState::new(evaluated_line.into_return_or_panic()))
            }
            VisitingCounting::Exit => {
                tracing::warn!("Visiting an Exit line does not require visiting state");
            }
        };
    }

    pub fn lines(&self, node_key: &NodeKey) -> Option<&Lines> {
        self.get(node_key).map(Deref::deref)
    }

    pub fn visiting_state<T: LineState>(&self, location: &Location) -> Option<&T> {
        self.visiting_counting(location)
            .and_then(|vc| match vc {
                VisitingCounting::Message(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Confirm(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Choice(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Eval(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Goto(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Call(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Return(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Exit => None,
            })
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn visiting_state_mut<T: LineState>(&mut self, location: &Location) -> Option<&mut T> {
        self.visiting_counting_mut(location)
            .and_then(|vc| match vc {
                VisitingCounting::Message(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Confirm(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Choice(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Eval(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Goto(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Call(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Return(s) => s.last_mut().map(|s| s.as_any_mut()),
                VisitingCounting::Exit => None,
            })
            .and_then(|v| v.downcast_mut::<T>())
    }

    fn visiting_counting(&self, location: &Location) -> Option<&VisitingCounting> {
        self.get(&location.node_key)
            .and_then(|node| node.get_index(*location.line_position).map(|(_, vc)| vc))
    }

    fn visiting_counting_mut(&mut self, location: &Location) -> Option<&mut VisitingCounting> {
        self.get_mut(&location.node_key).and_then(|node| {
            node.get_index_mut(*location.line_position)
                .map(|(_, vc)| vc)
        })
    }
}

impl Deref for VisitingStates {
    type Target = HashMap<NodeKey, Node>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct Node(Lines);

impl Deref for Node {
    type Target = Lines;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct Lines(IndexMap<LineId, VisitingCounting>);

impl Deref for Lines {
    type Target = IndexMap<LineId, VisitingCounting>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Lines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum VisitingCounting {
    Message(LineStates<MessageState>),
    Choice(LineStates<ChoiceState>),
    Confirm(LineStates<ConfirmState>),
    Eval(LineStates<EvalState>),
    Goto(LineStates<GotoState>),
    Call(LineStates<CallState>),
    Return(LineStates<ReturnState>),
    Exit,
}

impl VisitingCounting {
    pub fn is_visited(&self) -> bool {
        match self {
            VisitingCounting::Message(s) => !s.is_empty(),
            VisitingCounting::Confirm(s) => !s.is_empty(),
            VisitingCounting::Choice(s) => !s.is_empty(),
            VisitingCounting::Eval(s) => !s.is_empty(),
            VisitingCounting::Goto(s) => !s.is_empty(),
            VisitingCounting::Call(s) => !s.is_empty(),
            VisitingCounting::Return(s) => !s.is_empty(),
            VisitingCounting::Exit => false,
        }
    }

    pub fn visited_count(&self) -> usize {
        match self {
            VisitingCounting::Message(s) => s.len(),
            VisitingCounting::Confirm(s) => s.len(),
            VisitingCounting::Choice(s) => s.len(),
            VisitingCounting::Eval(s) => s.len(),
            VisitingCounting::Goto(s) => s.len(),
            VisitingCounting::Call(s) => s.len(),
            VisitingCounting::Return(s) => s.len(),
            VisitingCounting::Exit => 0,
        }
    }
}

impl From<&LineType> for VisitingCounting {
    fn from(line_type: &LineType) -> Self {
        match line_type {
            LineType::Message(_) => VisitingCounting::Message(MessageStates::default()),
            LineType::Confirm(_) => VisitingCounting::Confirm(ConfirmStates::default()),
            LineType::Choice(_) => VisitingCounting::Choice(ChoiceStates::default()),
            LineType::Eval(_) => VisitingCounting::Eval(EvalStates::default()),
            LineType::Goto(_) => VisitingCounting::Goto(GotoStates::default()),
            LineType::Call(_) => VisitingCounting::Call(CallStates::default()),
            LineType::Return(_) => VisitingCounting::Return(ReturnStates::default()),
            LineType::Exit(_) => VisitingCounting::Exit,
        }
    }
}

#[derive(Debug)]
pub struct LineStates<T: LineState>(Vec<T>);

impl<T: LineState> Default for LineStates<T> {
    fn default() -> Self {
        LineStates(Vec::new())
    }
}

impl<T: LineState> Deref for LineStates<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: LineState> DerefMut for LineStates<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

type MessageStates = LineStates<MessageState>;
type ConfirmStates = LineStates<ConfirmState>;
type ChoiceStates = LineStates<ChoiceState>;
type EvalStates = LineStates<EvalState>;
type GotoStates = LineStates<GotoState>;
type CallStates = LineStates<CallState>;
type ReturnStates = LineStates<ReturnState>;

impl ConfirmStates {
    pub fn confirmed(&self) -> Option<bool> {
        self.last().and_then(|s| s.confirmed)
    }
}

impl ChoiceStates {
    pub fn selected(&self) -> Option<&Selected> {
        self.last().and_then(|s| s.selected.as_ref())
    }
}

impl CallStates {
    pub fn returned_value(&self) -> Option<&JsValue> {
        self.last().and_then(|s| s.returned_value.as_ref())
    }
}
