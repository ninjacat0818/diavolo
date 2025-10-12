use rhai::Dynamic;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct DialogueCtx {
    args: Option<DialogueArgs>,
}

impl DialogueCtx {
    pub fn builder() -> DialogueCtxBuilder {
        DialogueCtxBuilder
    }

    pub fn new() -> Self {
        Self {
            // state: State::default(),
            // runner,
            args: None,
        }
    }
}

struct DialogueCtxBuilder;

impl DialogueCtxBuilder {
    pub fn args(self) -> Self {
        self
    }

    pub fn build(self) -> DialogueCtx {
        DialogueCtx::new()
    }
}

#[derive(Debug, Clone)]
struct DialogueArgs {
    immutable: HashMap<String, Dynamic>,
    mutable: HashMap<String, MutableVar>,
}

#[derive(Debug, Clone)]
struct MutableVar {
    value: Arc<Mutex<Dynamic>>,
}

impl MutableVar {
    pub fn new(value: Dynamic) -> Self {
        Self {
            value: Arc::new(Mutex::new(value)),
        }
    }

    pub fn register_in_rhai(&self, engine: &mut rhai::Engine, name: &str) {
        let value_ref = self.value.clone();
        let getter_name = format!("get_{}", name);
        engine.register_fn(&getter_name, move || -> Dynamic {
            value_ref.lock().unwrap().clone().into()
        });

        let value_ref = self.value.clone();
        let setter_name = format!("set_{}", name);
        engine.register_fn(&setter_name, move |new_val: Dynamic| {
            *value_ref.lock().unwrap() = new_val;
        });
    }
}

impl DialogueArgs {
    pub fn register_in_rhai(&self, engine: &mut rhai::Engine, scope: &mut rhai::Scope) {
        for (name, value) in &self.immutable {
            scope.push_constant_dynamic(name, value.clone());
        }

        for (name, mutable_var) in &self.mutable {
            mutable_var.register_in_rhai(engine, name);
        }
    }
}
