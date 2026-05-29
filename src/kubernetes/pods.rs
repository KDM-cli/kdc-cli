#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSummary {
    pub name: String,
    pub namespace: String,
    pub status: String,
}
