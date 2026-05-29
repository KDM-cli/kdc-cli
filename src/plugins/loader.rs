use crate::plugins::registry::PluginRegistry;

pub fn load_installed() -> PluginRegistry {
    PluginRegistry::default()
}
