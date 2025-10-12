use dialogue::{LinePosition, Location, NodeKey};

use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct CallStack(Vec<Location>);

impl Deref for CallStack {
    type Target = Vec<Location>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CallStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CallStack {
    pub fn call(&mut self, node_key: NodeKey) {
        const MAX_DEPTH: usize = 1024;
        if self.len() >= MAX_DEPTH {
            panic!("Maximum call stack depth ({}) exceeded.", MAX_DEPTH);
        }
        self.push(Location::uninitialized(node_key));
    }

    pub fn advance(&mut self) {
        self.goto(self.current_line_position().wrapping_add(1).into());
    }

    pub fn goto(&mut self, line_position: LinePosition) {
        self.last_mut().line_position = line_position;
    }

    pub fn location(&self) -> &Location {
        self.last()
    }

    pub fn current_node_key(&self) -> &NodeKey {
        &self.location().node_key
    }

    pub fn current_line_position(&self) -> LinePosition {
        self.location().line_position
    }

    fn last(&self) -> &Location {
        self.0.last().expect(EMPTY_ERROR)
    }

    fn last_mut(&mut self) -> &mut Location {
        self.0.last_mut().expect(EMPTY_ERROR)
    }
}

const EMPTY_ERROR: &str = "CallStack should have at least one element";
