pub(crate) mod config;
use config::Config;

#[derive(Debug, Default)]
pub struct Engine {
    pub(crate) config: Config,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
