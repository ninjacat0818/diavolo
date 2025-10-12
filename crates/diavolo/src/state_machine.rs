pub(crate) mod visiting_states;

use dialogue::{LineType, NodeKey};
use rhai::EvalAltResult;
use visiting_states::VisitingStates;
use visiting_states::visiting_state::VisitingState;

use std::ops::Add;
use std::time::Instant;
use std::usize;

#[derive(Debug)]
pub(super) struct StateMachine {
    pub execution_state: ExecutionState,
    pub node_key: NodeKey,
    pub line_position: usize,
    pub visiting_states: VisitingStates,
    pub fast_forward: Option<Instant>,
    rhai: Rhai,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self {
            execution_state: ExecutionState::default(),
            node_key: NodeKey::uninitialized(),
            line_position: usize::MAX,
            visiting_states: VisitingStates::default(),
            fast_forward: None,
            rhai: Rhai::default(),
        }
    }
}

#[derive(Debug, Default)]
struct Rhai {
    engine: rhai::Engine,
    scope: rhai::Scope<'static>,
}

impl Rhai {
    pub fn eval_if(
        &mut self,
        script: impl AsRef<str>,
    ) -> Result<bool, std::boxed::Box<EvalAltResult>> {
        self.engine
            .eval_expression_with_scope::<bool>(&mut self.scope, script.as_ref())
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

    pub fn visit_line(&mut self, line_type: &LineType) {
        tracing::debug!(
            "Visiting line: node={}, position={}",
            self.node_key,
            self.line_position
        );

        self.visiting_states.visit_line(
            &self.node_key,
            self.line_position,
            line_type,
            self.is_fast_forward(),
        );
    }

    pub fn is_initialidzed(&self) -> bool {
        !matches!(self.execution_state, ExecutionState::Uninitialized)
    }

    pub fn initialize(&mut self) {
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

    pub fn visiting_state<T: VisitingState + 'static>(&self) -> Option<&T> {
        self.visiting_states
            .visiting_state::<T>(&self.node_key, self.line_position)
    }

    pub fn visiting_state_mut<T: VisitingState + 'static>(&mut self) -> Option<&mut T> {
        self.visiting_states
            .visiting_state_mut::<T>(&self.node_key, self.line_position)
    }

    pub fn _visited_count(&self) -> usize {
        self.visiting_states
            ._visited_count(&self.node_key, self.line_position)
    }
}
