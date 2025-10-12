use super::runner::EvaluatedLine;
use super::visiting_state::VisitingState;
use super::visiting_state::choice::Selected;
use super::visiting_state::{
    CallVisitingState, ChoiceVisitingState, EvalVisitingState, GotoVisitingState,
    MessageVisitingState, ReturnVisitingState,
};
use boa_engine::JsValue;
use dialogue::{LineId, LineType, Location, NodeKey};

use indexmap::IndexMap;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct VisitingStates(HashMap<NodeKey, Node>);

impl VisitingStates {
    pub fn ensure_node(&mut self, node_key: NodeKey, node: &dialogue::Node) {
        if self.contains_key(&node_key) {
            tracing::debug!(
                "VisitingStates already initialized for node_key: {}",
                node_key
            );
            return;
        }

        tracing::debug!("Initializing VisitingStates for node_key: {}", node_key);

        let vs_node = self.entry(node_key).or_default();

        for line in node.iter() {
            let line_id = line.id.clone().unwrap_or_else(|| LineId::new_unique());
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
                states.push(MessageVisitingState::new(
                    initial_fast_forward,
                    evaluated_line.into_message(),
                ));
            }
            VisitingCounting::Choice(states) => {
                let (choice_texts, texts) = evaluated_line.into_choice();
                let state = match texts {
                    Some(texts) => ChoiceVisitingState::new_with_message(
                        choice_texts,
                        initial_fast_forward,
                        texts,
                    ),
                    None => ChoiceVisitingState::new(choice_texts),
                };
                states.push(state);
            }
            VisitingCounting::Eval(states) => {
                states.push(EvalVisitingState::new(evaluated_line.into_eval()));
            }
            VisitingCounting::Goto(states) => {
                states.push(GotoVisitingState::new(evaluated_line.into_goto()));
            }
            VisitingCounting::Call(states) => {
                states.push(CallVisitingState::new(evaluated_line.into_call()));
            }
            VisitingCounting::Return(states) => {
                states.push(ReturnVisitingState::new(evaluated_line.into_return()))
            }
            VisitingCounting::Exit => {
                tracing::warn!("Visiting an Exit line does not require visiting state");
            }
        };
    }

    pub fn lines(&self, node_key: &NodeKey) -> Option<&Lines> {
        self.get(node_key).map(Deref::deref)
    }

    pub fn visiting_state<T: VisitingState>(&self, location: &Location) -> Option<&T> {
        self.visiting_counting(location)
            .and_then(|vc| match vc {
                VisitingCounting::Message(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Choice(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Eval(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Goto(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Call(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Return(s) => s.last().map(|s| s.as_any()),
                VisitingCounting::Exit => None,
            })
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn visiting_state_mut<T: VisitingState>(&mut self, location: &Location) -> Option<&mut T> {
        self.visiting_counting_mut(location)
            .and_then(|vc| match vc {
                VisitingCounting::Message(s) => s.last_mut().map(|s| s.as_any_mut()),
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
    Message(MessageVisitingStates),
    Choice(ChoiceVisitingStates),
    Eval(EvalVisitingStates),
    Goto(GotoVisitingStates),
    Call(CallVisitingStates),
    Return(ReturnVisitingStates),
    Exit,
}

impl VisitingCounting {
    pub fn is_visited(&self) -> bool {
        match self {
            VisitingCounting::Message(s) => !s.is_empty(),
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
            LineType::Message(_) => VisitingCounting::Message(MessageVisitingStates::default()),
            LineType::Choice(_) => VisitingCounting::Choice(ChoiceVisitingStates::default()),
            LineType::Eval(_) => VisitingCounting::Eval(EvalVisitingStates::default()),
            LineType::Goto(_) => VisitingCounting::Goto(GotoVisitingStates::default()),
            LineType::Call(_) => VisitingCounting::Call(CallVisitingStates::default()),
            LineType::Return(_) => VisitingCounting::Return(ReturnVisitingStates::default()),
            LineType::Exit(_) => VisitingCounting::Exit,
            _ => panic!("Unsupported line type for VisitingCounting"),
        }
    }
}

#[derive(Debug, Default)]
pub struct MessageVisitingStates(Vec<MessageVisitingState>);

impl Deref for MessageVisitingStates {
    type Target = Vec<MessageVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct ChoiceVisitingStates(Vec<ChoiceVisitingState>);

impl ChoiceVisitingStates {
    pub fn selected(&self) -> Option<&Selected> {
        self.last().and_then(|state| state.selected.as_ref())
    }
}

impl Deref for ChoiceVisitingStates {
    type Target = Vec<ChoiceVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChoiceVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct EvalVisitingStates(Vec<EvalVisitingState>);

impl Deref for EvalVisitingStates {
    type Target = Vec<EvalVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EvalVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct GotoVisitingStates(Vec<GotoVisitingState>);

impl Deref for GotoVisitingStates {
    type Target = Vec<GotoVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GotoVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct CallVisitingStates(Vec<CallVisitingState>);

impl CallVisitingStates {
    pub fn returned_value(&self) -> Option<&JsValue> {
        self.last().and_then(|state| state.returned_value.as_ref())
    }
}

impl Deref for CallVisitingStates {
    type Target = Vec<CallVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CallVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct ReturnVisitingStates(Vec<ReturnVisitingState>);

impl Deref for ReturnVisitingStates {
    type Target = Vec<ReturnVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReturnVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
