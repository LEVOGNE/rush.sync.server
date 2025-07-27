// =====================================================
// VOLLSTÄNDIGER SYSTEM-TEST - MIT i18n INITIALISIERUNG
// =====================================================

use rush_sync_server::CommandHandler;

/// Hilfsfunktion: i18n für Tests initialisieren
async fn setup_test_environment() {
    // i18n initialisieren (wie in main.rs)
    let _ = rush_sync_server::i18n::init().await;
}

#[tokio::test]
async fn test_commands_with_real_translations() {
    // Setup
    setup_test_environment().await;
    let handler = CommandHandler::new();

    println!("🔍 TESTING WITH REAL TRANSLATIONS:");

    // Test: Version Command
    let result = handler.handle_input("version");
    println!("📝 Version: {}", result.message);
    assert!(result.success);
    assert!(
        result.message.contains("Rush Sync") || result.message.contains("Version"),
        "Should contain version info"
    );

    // Test: Language Command
    let result = handler.handle_input("lang");
    println!("📝 Language: {}", result.message);
    assert!(result.success);
    assert!(
        result.message.contains("Current")
            || result.message.contains("EN")
            || result.message.contains("Available"),
        "Should show language info"
    );

    // Test: Exit Command
    let result = handler.handle_input("exit");
    println!("📝 Exit: {}", result.message);
    assert!(result.success);
    assert!(
        result.message.contains("confirm")
            || result.message.contains("quit")
            || result.message.contains("y/n")
            || result.message.contains("j/n"),
        "Should ask for confirmation"
    );

    println!("✅ All commands work with real translations!");
}

#[tokio::test]
async fn test_language_switching() {
    setup_test_environment().await;
    let handler = CommandHandler::new();

    println!("🔍 TESTING LANGUAGE SWITCHING:");

    // Test: Switch to German
    let result = handler.handle_input_async("lang de").await;
    println!("📝 Switch to German: {}", result.message);
    assert!(result.success);

    // Test: Switch to English
    let result = handler.handle_input_async("lang en").await;
    println!("📝 Switch to English: {}", result.message);
    assert!(result.success);

    // Test: Current language status
    let result = handler.handle_input("lang");
    println!("📝 Current status: {}", result.message);
    assert!(result.success);

    println!("✅ Language switching works!");
}

#[test]
fn test_system_without_i18n() {
    // Teste dass System auch OHNE i18n funktioniert
    let handler = CommandHandler::new();

    // Commands sollten immer funktionieren, auch mit Warning-Messages
    assert!(handler.handle_input("version").success);
    assert!(handler.handle_input("clear").success);
    assert!(handler.handle_input("exit").success);
    assert!(!handler.handle_input("unknown").success);

    println!("✅ System works even without i18n initialization!");
}

#[test]
fn test_special_commands() {
    let handler = CommandHandler::new();

    // Test: Clear Command (sollte immer __CLEAR__ zurückgeben)
    let result = handler.handle_input("clear");
    assert!(result.success);
    assert_eq!(result.message, "__CLEAR__");

    // Test: Exit Command (sollte immer __CONFIRM_EXIT__ enthalten)
    let result = handler.handle_input("exit");
    assert!(result.success);
    assert!(result.message.starts_with("__CONFIRM_EXIT__"));

    println!("✅ Special command formats work correctly!");
}
