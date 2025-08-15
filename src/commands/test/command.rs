// =====================================================
// FILE: src/commands/test/command.rs
// =====================================================
use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct TestCommand;

impl Command for TestCommand {
    fn name(&self) -> &'static str {
        "test"
    }

    fn description(&self) -> &'static str {
        "Test command for debugging output"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("test")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => {
                // ✅ EINFACHER EINZEILIGER TEST
                Ok("🔥 TEST: Einzeiliger Text funktioniert!".to_string())
            }
            Some(&"multi") => {
                // ✅ MEHRZEILIGER TEST
                Ok(format!(
                    "🔥 TEST: Mehrzeiliger Text:\nZeile 1\nZeile 2\nZeile 3"
                ))
            }
            Some(&"long") => {
                // ✅ LANGER TEXT TEST
                Ok(format!(
                    "🔥 TEST: Sehr langer Text: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
                ))
            }
            Some(&"format") => {
                // ✅ FORMAT TEST (wie bei theme help)
                Ok(format!(
                    "🔥 TEST Format:\n\
                    Line 1: Hello World\n\
                    Line 2: This is a test\n\
                    Line 3: With multiple lines"
                ))
            }
            Some(&"emoji") => {
                // ✅ EMOJI TEST
                Ok("🎨🔥✅🎯🚀 Emoji Test funktioniert! 🎉".to_string())
            }
            Some(&"theme") => {
                // ✅ THEME DEBUG TEST
                Ok("🔥 TEST: Theme-Command wird aufgerufen - das funktioniert!".to_string())
            }
            Some(&"theme-help") => {
                // ✅ NACHBAU DES THEME HELP
                Ok(format!(
                    "🎨 TEST Theme Help:\nLine 1: theme\nLine 2: theme <name>\nLine 3: theme -h"
                ))
            }
            _ => Ok(
                "🔥 TEST Optionen: test, test multi, test long, test format, test emoji"
                    .to_string(),
            ),
        }
    }

    fn priority(&self) -> u8 {
        10 // Niedrige Priorität für Test-Command
    }

    fn is_available(&self) -> bool {
        cfg!(debug_assertions) // Nur in Debug-builds
    }
}
