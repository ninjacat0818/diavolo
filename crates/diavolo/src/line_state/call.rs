use super::LineState;

use boa_engine::JsValue;
use dialogue::NodeKey;
use std::time::Instant;

impl LineState for CallState {
    fn visited_at(&self) -> Instant {
        self.visited_at
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct CallState {
    pub visited_at: Instant,
    pub node_key: dialogue::NodeKey,
    pub returned_value: Option<JsValue>,
    pub returned_at: Option<Instant>,
}

impl CallState {
    pub fn new(node_key: NodeKey) -> Self {
        Self {
            visited_at: Instant::now(),
            node_key,
            returned_at: None,
            returned_value: None,
        }
    }

    pub fn _is_returned(&self) -> bool {
        self.returned_at.is_some()
    }

    pub fn ret(&mut self, value: JsValue) {
        if self.returned_at.is_some() {
            tracing::warn!("Call already returned");
            return;
        }
        self.returned_at.replace(Instant::now());
        self.returned_value.replace(value);
    }
}
