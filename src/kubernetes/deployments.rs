#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentSummary {
    pub name: String,
    pub namespace: String,
    pub ready: String,
}
