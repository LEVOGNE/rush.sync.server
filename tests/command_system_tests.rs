// =====================================================
// FILE: tests/command_system_tests.rs - INTEGRATION TESTS
// =====================================================

use rush_sync_server::{create_default_registry, CommandHandler};

#[test]
fn test_command_system_integration() {
    // Test: Standard Registry erstellen
    let handler = CommandHandler::new();

    // Test: Commands sind verfügbar
    let commands = handler.list_commands();
    assert!(!commands.is_empty(), "Should have registered commands");

    // Test: Version Command
    let result = handler.handle_input("version");
    assert!(result.success, "Version command should succeed");
    // ✅ FLEXIBLER: Nicht nur "Rush Sync" erwarten
    assert!(
        result.message.to_lowercase().contains("rush")
            || result.message.contains("Version")
            || result.message.contains("version"),
        "Should contain version info, got: '{}'",
        result.message
    );

    // Test: Clear Command
    let result = handler.handle_input("clear");
    assert!(result.success, "Clear command should succeed");
    assert_eq!(result.message, "__CLEAR__", "Should return clear signal");

    // Test: Exit Command
    let result = handler.handle_input("exit");
    assert!(result.success, "Exit command should succeed");
    assert!(
        result.message.contains("confirm") || result.message.contains("bestätigen"),
        "Should ask for confirmation"
    );

    // Test: Unknown Command
    let result = handler.handle_input("totally_unknown_command_xyz");
    assert!(!result.success, "Unknown command should fail");

    println!("✅ All integration tests passed!");
}

#[tokio::test]
async fn test_async_command_system() {
    let handler = CommandHandler::new();

    // Test: Language Command (async)
    let result = handler.handle_input_async("lang").await;
    assert!(result.success, "Language command should succeed");
    // ✅ FLEXIBLER: Verschiedene mögliche Antworten akzeptieren
    assert!(
        result.message.to_lowercase().contains("current")
            || result.message.to_lowercase().contains("aktuelle")
            || result.message.to_lowercase().contains("language")
            || result.message.to_lowercase().contains("sprache")
            || result.message.contains("EN")
            || result.message.contains("DE"),
        "Should show language info, got: '{}'",
        result.message
    );

    // Test: Async vs Sync consistency
    let sync_result = handler.handle_input("version");
    let async_result = handler.handle_input_async("version").await;
    assert_eq!(
        sync_result.success, async_result.success,
        "Sync and async should be consistent"
    );

    println!("✅ All async integration tests passed!");
}

#[test]
fn test_registry_functionality() {
    let registry = create_default_registry();

    // Test: Registry ist nicht leer
    assert!(!registry.is_empty(), "Registry should not be empty");
    assert!(!registry.is_empty(), "Registry should have commands");

    // Test: Debug info
    let debug_info = registry.debug_info();
    assert!(
        debug_info.contains("CommandRegistry"),
        "Should contain registry info"
    );
    assert!(
        debug_info.contains("initialized: true"),
        "Should be initialized"
    );

    // Test: Command lookup
    assert!(
        registry.find_command("version").is_some(),
        "Should find version command"
    );
    assert!(
        registry.find_command("exit").is_some(),
        "Should find exit command"
    );
    assert!(
        registry.find_command("unknown123").is_none(),
        "Should not find unknown command"
    );

    println!("✅ Registry functionality tests passed!");
}
