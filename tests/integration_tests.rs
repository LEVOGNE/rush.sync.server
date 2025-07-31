// ## FILE: tests/integration_tests.rs (KONSOLIDIERT)
// ## BEGIN ##
use rush_sync_server::{create_default_registry, CommandHandler};

#[test]
fn test_core_functionality() {
    let handler = CommandHandler::new();

    // Basis-Funktionalität
    assert!(!handler.list_commands().is_empty());

    // Commands testen
    let test_cases = [
        ("version", true),
        ("clear", true),
        ("exit", true),
        ("unknown_xyz", false),
    ];

    for (command, should_succeed) in test_cases {
        let result = handler.handle_input(command);
        assert_eq!(
            result.success, should_succeed,
            "Command '{}' failed",
            command
        );
    }
}

#[tokio::test]
async fn test_async_commands() {
    let handler = CommandHandler::new();
    let result = handler.handle_input_async("lang").await;
    assert!(result.success);
}

#[test]
fn test_registry() {
    let registry = create_default_registry();
    assert!(!registry.is_empty());
    assert!(registry.find_command("version").is_some());
}
// ## END ##
