// =====================================================
// FILE: tests/simple_test.rs - EINFACHER FUNKTIONSTEST
// =====================================================

use rush_sync_server::CommandHandler;

#[test]
fn test_basic_functionality() {
    // Test: Handler erstellen
    let handler = CommandHandler::new();
    println!("✅ CommandHandler created successfully");

    // Test: Version Command
    let result = handler.handle_input("version");
    assert!(result.success, "Version command should work");
    println!("✅ Version command: {}", result.message);

    // Test: Commands auflisten
    let commands = handler.list_commands();
    assert!(!commands.is_empty(), "Should have commands");
    println!("✅ Available commands: {}", commands.len());

    // Test: Debug Info
    let debug = handler.debug_info();
    assert!(debug.contains("CommandRegistry"), "Should have debug info");
    println!("✅ Debug info: {}", debug);

    println!("🎯 Basic functionality test passed!");
}

#[test]
fn test_command_execution() {
    let handler = CommandHandler::new();

    // Test verschiedene Commands
    let test_cases = vec![
        ("version", true),
        ("clear", true),
        ("exit", true),
        ("unknown_xyz", false),
    ];

    for (command, should_succeed) in test_cases {
        let result = handler.handle_input(command);
        assert_eq!(
            result.success, should_succeed,
            "Command '{}' success should be {}",
            command, should_succeed
        );
        println!(
            "✅ Command '{}': {}",
            command,
            if result.success { "✓" } else { "✗" }
        );
    }

    println!("🎯 Command execution test passed!");
}
