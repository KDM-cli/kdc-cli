use crate::plugins::registry::PluginRegistry;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginManager {
    pub registry: PluginRegistry,
}

impl PluginManager {
    pub fn load() -> Self {
        Self {
            registry: crate::plugins::loader::load_installed(),
        }
    }

    pub fn capabilities(&self) -> Vec<&str> {
        self.registry
            .plugins
            .iter()
            .flat_map(|plugin| plugin.capabilities.iter().map(String::as_str))
            .collect()
    }
}
