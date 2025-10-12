pub(super) mod config;

use config::Config;
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
}

impl Engine {
    pub fn with_config(config: Config) -> Self {
        Self {
            inner: Arc::new(EngineInner { config }),
        }
    }

    pub fn config(&self) -> &Config {
        self.inner.config()
    }

    pub fn config_mut(&mut self) -> Option<&mut Config> {
        Arc::get_mut(&mut self.inner).map(|inner| inner.config_mut())
    }
}

#[derive(Debug, Default)]
struct EngineInner {
    config: Config,
}

impl EngineInner {
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
