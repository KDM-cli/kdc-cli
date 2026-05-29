#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesEvent {
    pub reason: String,
    pub message: String,
}
