// =====================================================
// DEBUG TEST - Schauen was die Commands wirklich zurückgeben
// =====================================================

use rush_sync::CommandHandler;

#[test]
fn debug_command_outputs() {
    let handler = CommandHandler::new();

    println!("🔍 DEBUGGING COMMAND OUTPUTS:");

    // Test: Version Command
    let result = handler.handle_input("version");
    println!("📝 Version command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);
    println!(
        "   Contains 'Rush Sync': {}",
        result.message.contains("Rush Sync")
    );
    println!(
        "   Contains 'rush': {}",
        result.message.to_lowercase().contains("rush")
    );

    // Test: Language Command
    let result = handler.handle_input("lang");
    println!("📝 Language command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);
    println!(
        "   Contains 'Current': {}",
        result.message.contains("Current")
    );
    println!(
        "   Contains 'Aktuelle': {}",
        result.message.contains("Aktuelle")
    );
    println!(
        "   Contains 'language': {}",
        result.message.to_lowercase().contains("language")
    );

    // Test: Clear Command
    let result = handler.handle_input("clear");
    println!("📝 Clear command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);

    // Test: Exit Command
    let result = handler.handle_input("exit");
    println!("📝 Exit command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);

    // Test: Unknown Command
    let result = handler.handle_input("unknown123");
    println!("📝 Unknown command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);

    println!("🎯 Debug complete!");
}

#[tokio::test]
async fn debug_async_commands() {
    let handler = CommandHandler::new();

    println!("🔍 DEBUGGING ASYNC COMMANDS:");

    // Test: Async Language Command
    let result = handler.handle_input_async("lang").await;
    println!("📝 Async Language command:");
    println!("   Success: {}", result.success);
    println!("   Message: '{}'", result.message);

    println!("🎯 Async debug complete!");
}
