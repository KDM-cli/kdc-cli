use crate::plugins::registry::PluginRegistry;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginManager {
    pub registry: PluginRegistry,
}
