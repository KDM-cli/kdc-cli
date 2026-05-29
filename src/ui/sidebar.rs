use crate::domain::menu::MenuItem;

pub fn visible_labels(items: &[MenuItem]) -> Vec<&str> {
    items
        .iter()
        .filter(|item| item.visible)
        .map(|item| item.label.as_str())
        .collect()
}
