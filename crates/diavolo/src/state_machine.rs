pub mod call_stack;

use call_stack::CallStack;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct StateMachine {
    pub call_stack: CallStack,
    pub fast_forward: Option<Instant>,
}

impl Deref for StateMachine {
    type Target = CallStack;

    fn deref(&self) -> &Self::Target {
        &self.call_stack
    }
}

impl DerefMut for StateMachine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.call_stack
    }
}

impl StateMachine {
    pub fn is_fast_forward(&self) -> bool {
        self.fast_forward.is_some()
    }
}

use dialogue::{Line, LineIf, LineType, Node, Nodes};

impl<'d> StateMachine {
    pub fn is_last_line(&self, nodes: &Nodes) -> bool {
        self.current_line_position().wrapping_add(1)
            == self
                .current_node(nodes)
                .map(|node| node.len())
                .unwrap_or(usize::MAX)
    }

    pub fn current_line_type(&self, nodes: &'d Nodes) -> Option<&'d LineType> {
        self.current_line(nodes).map(|line| &line.r#type)
    }

    pub fn current_line_if(&self, nodes: &'d Nodes) -> Option<&'d LineIf> {
        self.current_line(nodes).and_then(|line| line.r#if.as_ref())
    }

    fn current_line(&self, nodes: &'d Nodes) -> Option<&'d Line> {
        self.current_node(nodes)
            .and_then(|node| node.get(*self.call_stack.current_line_position()))
    }

    fn current_node(&self, nodes: &'d Nodes) -> Option<&'d Node> {
        nodes.get(self.call_stack.current_node_key())
    }
}
