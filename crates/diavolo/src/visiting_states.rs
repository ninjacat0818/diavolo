use super::visiting_state::choice::Selected;
use super::visiting_state::{ChoiceVisitingState, MessageVisitingState, OtherVisitingState};
use dialogue::{ChoiceTexts, LangTexts, LineId, LineType, NodeKey};

use boa_engine::JsResult;
use indexmap::IndexMap;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct VisitingStates(HashMap<NodeKey, Node>);

impl VisitingStates {
    pub fn init_node(&mut self, node_key: &NodeKey, node: &dialogue::Node) {
        tracing::debug!("Initializing VisitingStates for node_key: {}", node_key);

        let vs_node = self.entry(node_key.clone()).or_default();

        for line in node.iter() {
            let line_id = line.id.clone().unwrap_or_else(|| LineId::new_unique());
            let vc = match line.r#type {
                LineType::Message(_) => VisitingCounting::Message(MessageVisitingStates::default()),
                LineType::Choice(_) => VisitingCounting::Choice(ChoiceVisitingStates::default()),
                LineType::Confirm(_) => VisitingCounting::Other(OtherVisitingStates::default()),
            };
            vs_node.insert(line_id, vc);
        }
    }

    pub fn visit_line(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
        initial_fast_forward: bool,
        evaluated_texts: Option<LangTexts>,
        evaluated_choice_texts: Option<ChoiceTexts>,
    ) -> JsResult<()> {
        let visiting_counting = self
            .get_mut(node_key)
            .expect("Node must be initialized before visiting lines")
            .get_index_mut(line_position)
            .map(|(_, vc)| vc)
            .expect("Line must be initialized before visiting");

        match visiting_counting {
            VisitingCounting::Message(states) => {
                states.push(MessageVisitingState::new(
                    initial_fast_forward,
                    evaluated_texts.unwrap(),
                ));
            }
            VisitingCounting::Choice(states) => {
                let evaluated_choice_texts = evaluated_choice_texts
                    .expect("evaluated_choice_texts is required for Choice line");
                let state = match evaluated_texts {
                    Some(evaluated_texts) => ChoiceVisitingState::new_with_message(
                        evaluated_choice_texts,
                        initial_fast_forward,
                        evaluated_texts,
                    ),
                    None => ChoiceVisitingState::new(evaluated_choice_texts),
                };
                states.push(state);
            }
            VisitingCounting::Other(states) => states.push(OtherVisitingState {}),
        };
        Ok(())
    }

    pub fn lines(&self, node_key: &NodeKey) -> Option<&Lines> {
        self.get(node_key).map(Deref::deref)
    }

    pub fn message_visiting_state(
        &self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&MessageVisitingState> {
        self.visiting_counting(node_key, line_position)
            .and_then(|vc| match vc {
                VisitingCounting::Message(states) => states.last(),
                _ => None,
            })
    }

    pub fn message_visiting_state_mut(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&mut MessageVisitingState> {
        self.visiting_counting_mut(node_key, line_position)
            .and_then(|vc| match vc {
                VisitingCounting::Message(states) => states.last_mut(),
                _ => None,
            })
    }

    pub fn choice_visiting_state(
        &self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&ChoiceVisitingState> {
        self.visiting_counting(node_key, line_position)
            .and_then(|vc| match vc {
                VisitingCounting::Choice(states) => states.last(),
                _ => None,
            })
    }

    pub fn choice_visiting_state_mut(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&mut ChoiceVisitingState> {
        self.visiting_counting_mut(node_key, line_position)
            .and_then(|vc| match vc {
                VisitingCounting::Choice(states) => states.last_mut(),
                _ => None,
            })
    }

    pub fn _visited_count(&self, node_key: &NodeKey, line_position: usize) -> usize {
        self.visiting_counting(node_key, line_position)
            .map(|visiting_counting| match visiting_counting {
                VisitingCounting::Message(states) => states.len(),
                VisitingCounting::Choice(states) => states.len(),
                VisitingCounting::Other(states) => states.len(),
            })
            .unwrap_or_default()
    }

    fn visiting_counting(
        &self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&VisitingCounting> {
        self.get(node_key)
            .and_then(|node| node.get_index(line_position).map(|(_, vc)| vc))
    }

    fn visiting_counting_mut(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&mut VisitingCounting> {
        self.get_mut(node_key)
            .and_then(|node| node.get_index_mut(line_position).map(|(_, vc)| vc))
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
    Other(OtherVisitingStates),
}

impl VisitingCounting {
    pub fn is_visited(&self) -> bool {
        match self {
            VisitingCounting::Message(states) => !states.is_empty(),
            VisitingCounting::Choice(states) => !states.is_empty(),
            VisitingCounting::Other(states) => !states.is_empty(),
        }
    }

    pub fn visited_count(&self) -> usize {
        match self {
            VisitingCounting::Message(states) => states.len(),
            VisitingCounting::Choice(states) => states.len(),
            VisitingCounting::Other(states) => states.len(),
        }
    }

    pub fn is_selected(&self) -> Option<bool> {
        match self {
            VisitingCounting::Choice(states) => states.last().map(|state| state.is_selected()),
            _ => None,
        }
    }

    pub fn selected(&self) -> Option<&Selected> {
        match self {
            VisitingCounting::Choice(states) => {
                states.last().and_then(|state| state.selected.as_ref())
            }
            _ => None,
        }
    }

    pub fn get_selected(&self, index: usize) -> Option<&Selected> {
        match self {
            VisitingCounting::Choice(states) => {
                states.get(index).and_then(|state| state.selected.as_ref())
            }
            _ => None,
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
pub struct OtherVisitingStates(Vec<OtherVisitingState>);

impl Deref for OtherVisitingStates {
    type Target = Vec<OtherVisitingState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OtherVisitingStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
