pub(crate) mod visiting_state;

use dialogue::{LineType, NodeKey};
use visiting_state::{
    ChoiceVisitingState, MessageVisitingState, OtherVisitingState, VisitingState,
};

use std::any::Any;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct VisitingStates(HashMap<NodeKey, Node>);

impl VisitingStates {
    pub fn visit_line(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
        line_type: &LineType,
        initial_fast_forward: bool,
    ) {
        let node = self.entry(node_key.clone()).or_default();
        let visiting_counting = node
            .entry(line_position)
            .or_insert_with(|| match line_type {
                LineType::Message(_) => VisitingCounting::Message(vec![]),
                LineType::Choice(_) => VisitingCounting::Choice(vec![]),
                LineType::Confirm(_) => VisitingCounting::Other(vec![]),
            });
        match visiting_counting {
            VisitingCounting::Message(states) => {
                states.push(MessageVisitingState::new(initial_fast_forward))
            }
            VisitingCounting::Choice(states) => match line_type {
                LineType::Choice(choice) => states.push(
                    choice
                        .has_message()
                        .then(|| ChoiceVisitingState::new_with_message(initial_fast_forward))
                        .unwrap_or(ChoiceVisitingState::new()),
                ),
                _ => unreachable!(),
            },
            VisitingCounting::Other(states) => states.push(OtherVisitingState {}),
        };
    }

    pub fn visiting_state<T: VisitingState + 'static>(
        &self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&T> {
        let state: Option<&dyn Any> = match self.visiting_counting(node_key, line_position) {
            VisitingCounting::Message(states) => states.last().map(|s| s as &dyn Any),
            VisitingCounting::Choice(states) => states.last().map(|s| s as &dyn Any),
            VisitingCounting::Other(states) => states.last().map(|s| s as &dyn Any),
        };
        state.and_then(|s| s.downcast_ref::<T>())
    }

    pub fn visiting_state_mut<T: VisitingState + 'static>(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> Option<&mut T> {
        let state: Option<&mut dyn Any> = match self.visiting_counting_mut(node_key, line_position)
        {
            VisitingCounting::Message(states) => states.last_mut().map(|s| s as &mut dyn Any),
            VisitingCounting::Choice(states) => states.last_mut().map(|s| s as &mut dyn Any),
            VisitingCounting::Other(states) => states.last_mut().map(|s| s as &mut dyn Any),
        };
        state.and_then(|s| s.downcast_mut::<T>())
    }

    pub fn _visited_count(&self, node_key: &NodeKey, line_position: usize) -> usize {
        match self.visiting_counting(node_key, line_position) {
            VisitingCounting::Message(states) => states.len(),
            VisitingCounting::Choice(states) => states.len(),
            VisitingCounting::Other(states) => states.len(),
        }
    }

    fn visiting_counting(&self, node_key: &NodeKey, line_position: usize) -> &VisitingCounting {
        let node = self.get(node_key).unwrap();
        node.get(&line_position).unwrap()
    }

    fn visiting_counting_mut(
        &mut self,
        node_key: &NodeKey,
        line_position: usize,
    ) -> &mut VisitingCounting {
        let node = self.get_mut(node_key).unwrap();
        node.get_mut(&line_position).unwrap()
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
pub struct Lines(HashMap<usize, VisitingCounting>);

impl Deref for Lines {
    type Target = HashMap<usize, VisitingCounting>;

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
    Message(Vec<MessageVisitingState>),
    Choice(Vec<ChoiceVisitingState>),
    Other(Vec<OtherVisitingState>),
}
