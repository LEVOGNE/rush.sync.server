use crate::core::prelude::*;

#[async_trait::async_trait]
pub trait Command: Send + Sync + std::fmt::Debug + 'static {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn matches(&self, command: &str) -> bool;

    // Hauptausführung - immer implementieren
    async fn execute(&self, args: &[&str]) -> Result<String> {
        // Einfach & robust: vorhandene Sync-Logik nutzen
        self.execute_sync(args)
    }

    // Optional: Sync-Fallback für Commands die es brauchen
    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        // Default: Blockiert auf async execute
        futures::executor::block_on(self.execute(args))
    }

    fn priority(&self) -> u8 {
        50
    }
    fn is_available(&self) -> bool {
        true
    }
}
