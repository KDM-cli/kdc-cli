use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRegistry {
    pub plugins: Vec<PluginDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDefinition {
    pub name: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub menus: Vec<PluginMenu>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMenu {
    pub id: String,
    pub label: String,
    #[serde(default = "default_screen")]
    pub screen: String,
}

fn default_screen() -> String {
    "settings".to_string()
}

impl PluginRegistry {
    pub fn names(&self) -> Vec<&str> {
        self.plugins
            .iter()
            .map(|plugin| plugin.name.as_str())
            .collect()
    }

    pub fn register(&mut self, plugin: PluginDefinition) {
        self.plugins.retain(|existing| existing.name != plugin.name);
        self.plugins.push(plugin);
        self.plugins
            .sort_by(|left, right| left.name.cmp(&right.name));
    }
}
