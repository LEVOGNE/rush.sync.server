use crate::core::prelude::*;

#[async_trait::async_trait]
pub trait Command: Send + Sync + std::fmt::Debug + 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn matches(&self, command: &str) -> bool;

    // Async execution - default calls sync version
    async fn execute(&self, args: &[&str]) -> Result<String> {
        self.execute_sync(args)
    }

    // Sync execution - MUST be implemented by commands that don't override execute()
    fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Err(crate::core::error::AppError::Validation(
            "Command must implement either execute() or execute_sync()".to_string(),
        ))
    }

    fn priority(&self) -> u8 {
        50
    }
    fn is_available(&self) -> bool {
        true
    }
}
