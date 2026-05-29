#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KubernetesStatus {
    Unknown,
    Connected,
    Disconnected,
}
