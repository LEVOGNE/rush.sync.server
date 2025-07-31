use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

pub trait Command: Send + Sync + std::fmt::Debug + 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn matches(&self, command: &str) -> bool;
    fn execute_sync(&self, args: &[&str]) -> Result<String>;

    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move { self.execute_sync(args) })
    }

    fn supports_async(&self) -> bool {
        false
    }
    fn priority(&self) -> u8 {
        50
    }
    fn is_available(&self) -> bool {
        true
    }
}
