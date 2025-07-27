// =====================================================
// FILE: commands/command.rs - EINFACHES OBJECT-SAFE TRAIT
// =====================================================

use crate::core::prelude::*;
use std::future::Future;
use std::pin::Pin;

/// ✅ OBJECT-SAFE Command Trait - Box<dyn Command> funktioniert perfekt!
pub trait Command: Send + Sync + std::fmt::Debug + 'static {
    /// Command Name für Registry
    fn name(&self) -> &'static str;

    /// Command Beschreibung
    fn description(&self) -> &'static str;

    /// Prüft ob Command matched
    fn matches(&self, command: &str) -> bool;

    /// Synchrone Ausführung
    fn execute_sync(&self, args: &[&str]) -> Result<String>;

    /// ✅ OBJECT-SAFE: Asynchrone Ausführung mit Pin<Box<...>>
    fn execute_async<'a>(
        &'a self,
        args: &'a [&'a str],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        // Default: Ruft sync Version auf
        Box::pin(async move { self.execute_sync(args) })
    }

    /// Unterstützt async?
    fn supports_async(&self) -> bool {
        false
    }

    /// Priorität für Command-Matching (höher = wird zuerst geprüft)
    fn priority(&self) -> u8 {
        50
    }

    /// Ist Command verfügbar?
    fn is_available(&self) -> bool {
        true
    }
}
