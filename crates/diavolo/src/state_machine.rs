use dialogue::NodeKey;

use std::ops::Add;
use std::time::Instant;
use std::usize;

#[derive(Debug)]
pub struct StateMachine {
    pub execution_state: ExecutionState,
    pub node_key: NodeKey,
    pub line_position: usize,
    pub fast_forward: Option<Instant>,
}

impl StateMachine {
    pub fn enter_fast_forward(&mut self) {
        self.fast_forward.replace(std::time::Instant::now());
    }

    pub fn leave_fast_forward(&mut self) {
        self.fast_forward.take();
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self {
            execution_state: ExecutionState::default(),
            node_key: NodeKey::uninitialized(),
            line_position: usize::MAX,
            fast_forward: None,
        }
    }
}

#[derive(Debug, Default)]
pub enum ExecutionState {
    #[default]
    Uninitialized,
    Running,
    Terminated,
}

impl StateMachine {
    pub fn change_node(&mut self, node_key: &NodeKey) {
        self.transition(Some(node_key), 0);
    }

    pub fn advance(&mut self) {
        self.transition(None, self.line_position.add(1));
    }

    pub fn transition(&mut self, node_key: Option<&NodeKey>, line_position: usize) {
        if let Some(node_key) = node_key {
            self.node_key = node_key.clone();
        }
        self.line_position = line_position;
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self.execution_state, ExecutionState::Uninitialized)
    }

    pub fn initialize(&mut self) {
        tracing::debug!("Initializing StateMachine");
        self.transition(Some(&NodeKey::main()), 0);
        self.execution_state = ExecutionState::Running;
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self.execution_state, ExecutionState::Terminated)
    }

    pub fn terminate(&mut self) {
        self.execution_state = ExecutionState::Terminated;
    }

    pub fn is_fast_forward(&self) -> bool {
        self.fast_forward.is_some()
    }
}

use dialogue::{Line, LineIf, LineType, Node, Nodes, TypingSpeed};

impl<'d> StateMachine {
    pub(super) fn is_last_line(&self, nodes: &Nodes) -> bool {
        self.line_position + 1 == self.current_node(nodes).len()
    }

    pub(super) fn current_line_type(&self, nodes: &'d Nodes) -> &'d LineType {
        &self.current_line(nodes).r#type
    }

    pub(super) fn current_line_if(&self, nodes: &'d Nodes) -> Option<&'d LineIf> {
        self.current_line(nodes).r#if.as_ref()
    }

    pub(super) fn current_line_speed(&self, nodes: &'d Nodes) -> Option<&'d TypingSpeed> {
        match self.current_line_type(nodes) {
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

    fn current_line(&self, nodes: &'d Nodes) -> &'d Line {
        &self.current_node(nodes)[self.line_position]
    }

    fn current_node(&self, nodes: &'d Nodes) -> &'d Node {
        &nodes[&self.node_key]
    }
}
