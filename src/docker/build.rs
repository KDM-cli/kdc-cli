#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequest {
    pub image: String,
    pub tag: String,
}
