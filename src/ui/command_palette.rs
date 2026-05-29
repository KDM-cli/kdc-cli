use crate::{commands::palette::CommandAction, domain::menu::MenuItem};

pub fn search<'a>(items: &'a [MenuItem], query: &str) -> Vec<&'a MenuItem> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return items.iter().filter(|item| item.enabled).collect();
    }

    items
        .iter()
        .filter(|item| item.enabled && item.label.to_lowercase().contains(&query))
        .collect()
}

pub fn search_actions<'a>(items: &'a [CommandAction], query: &str) -> Vec<&'a CommandAction> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return items.iter().filter(|item| item.enabled).collect();
    }

    items
        .iter()
        .filter(|item| {
            item.enabled
                && (item.label.to_lowercase().contains(&query)
                    || item.id.to_lowercase().contains(&query))
        })
        .collect()
}
