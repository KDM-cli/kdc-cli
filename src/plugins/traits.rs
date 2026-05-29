use crate::domain::menu::MenuItem;

pub trait Plugin {
    fn name(&self) -> &str;
    fn register(&self) -> Vec<MenuItem>;
}
