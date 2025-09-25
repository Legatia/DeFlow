/// Test modules for DeFlow backend functionality
///
/// This module contains comprehensive tests for various components
/// of the DeFlow backend system, including social media integrations,
/// DeFi protocols, workflow execution, and more.

// Social media integration tests (basic version)
pub mod social_media_basic_tests;

// Re-export test utilities for easy access
pub use social_media_basic_tests::social_test_utilities::{
    create_test_telegram_alert,
    create_test_discord_alert,
    create_test_multi_platform_alert,
    create_test_token_price,
    validate_alert_structure,
};

#[cfg(test)]
mod test_config {
    /// Global test configuration and setup

    // Test timeout in milliseconds
    pub const DEFAULT_TEST_TIMEOUT: u64 = 5000;

    // Mock API endpoints for testing
    pub const MOCK_TELEGRAM_API: &str = "https://api.telegram.org/bot";
    pub const MOCK_DISCORD_WEBHOOK: &str = "https://discord.com/api/webhooks";

    // Test data constants
    pub const TEST_USER_ID: &str = "test_user_123";
    pub const TEST_CHAT_ID: &str = "test_chat_456";
    pub const TEST_BOT_TOKEN: &str = "test_bot_token_789";
}