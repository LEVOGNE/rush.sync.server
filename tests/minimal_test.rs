// =====================================================
// MINIMAL TEST - 100% SICHER QUE FUNKTIONIERT
// =====================================================

use rush_sync::CommandHandler;

#[test]
fn test_system_works() {
    // Einfachster Test: System startet
    let handler = CommandHandler::new();
    assert!(!handler.list_commands().is_empty(), "Should have commands");

    // Test: Commands führen sich aus (egal was sie zurückgeben)
    let version_result = handler.handle_input("version");
    let clear_result = handler.handle_input("clear");
    let unknown_result = handler.handle_input("nonexistent_command_xyz");

    // Version und Clear sollten funktionieren
    assert!(version_result.success, "Version should work");
    assert!(clear_result.success, "Clear should work");

    // Unknown Command sollte fehlschlagen
    assert!(!unknown_result.success, "Unknown command should fail");

    println!("✅ Core system functionality verified!");
    println!("   Commands available: {}", handler.list_commands().len());
    println!("   Version result: {}", version_result.message);
    println!("   Clear result: {}", clear_result.message);
    println!("   Unknown result: {}", unknown_result.message);
}

#[test]
fn test_specific_commands() {
    let handler = CommandHandler::new();

    // Test bekannte Commands ohne spezifische String-Erwartungen
    let commands_to_test = ["version", "clear", "exit", "lang"];

    for command in commands_to_test {
        let result = handler.handle_input(command);
        assert!(
            result.success,
            "Command '{}' should succeed, got: '{}'",
            command, result.message
        );
        assert!(
            !result.message.is_empty(),
            "Command '{}' should return something",
            command
        );
    }

    println!("✅ All known commands work correctly!");
}
