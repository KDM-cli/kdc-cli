use crate::domain::project::ProjectStack;

pub fn build_command(stack: ProjectStack) -> Option<&'static str> {
    match stack {
        ProjectStack::Node => Some("npm run build"),
        ProjectStack::Spring => Some("mvn package"),
        ProjectStack::Rust => Some("cargo build"),
        ProjectStack::Go => Some("go build ./..."),
        ProjectStack::Python => Some("python -m compileall ."),
        ProjectStack::Unknown => None,
    }
}
