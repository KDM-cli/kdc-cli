#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerStatus {
    Unknown,
    Running,
    Unavailable,
}
