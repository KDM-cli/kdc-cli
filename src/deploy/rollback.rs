#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRequest {
    pub target_revision: Option<String>,
}
