#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeRestartRequest {
    pub service: Option<String>,
}
