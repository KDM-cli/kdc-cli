#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerImage {
    pub repository: String,
    pub tag: String,
}
