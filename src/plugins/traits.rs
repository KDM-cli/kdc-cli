use crate::domain::menu::MenuItem;

pub trait Plugin {
    fn name(&self) -> &str;
    fn register(&self) -> Vec<MenuItem>;
}

pub fn menu_items_from_registry(
    registry: &crate::plugins::registry::PluginRegistry,
) -> Vec<MenuItem> {
    registry
        .plugins
        .iter()
        .flat_map(|plugin| {
            plugin.menus.iter().map(|menu| {
                MenuItem::visible(
                    &menu.id,
                    &menu.label,
                    screen_from_plugin(&menu.screen),
                    None,
                )
            })
        })
        .collect()
}

fn screen_from_plugin(screen: &str) -> crate::domain::screen::Screen {
    match screen {
        "docker" => crate::domain::screen::Screen::Docker,
        "compose" => crate::domain::screen::Screen::Compose,
        "kubernetes" => crate::domain::screen::Screen::Kubernetes,
        "monitoring" => crate::domain::screen::Screen::Monitoring,
        "deployments" => crate::domain::screen::Screen::Deployments,
        _ => crate::domain::screen::Screen::Settings,
    }
}
