use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub message: String,
}

pub trait CommandExecutor {
    fn execute(&self, action_id: &str) -> Result<ExecutionResult>;
}
