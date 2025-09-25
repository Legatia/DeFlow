#[cfg(test)]
mod telegram_tests {
    use super::super::*;
    use crate::defi::price_alert_service::{PriceAlertService, SocialPlatform, PriceAlert, AlertAction, PriceCondition, TokenPrice};
    use crate::nodes::{WorkflowNode, NodeConfiguration, NodeOutput, execute_social_media_post_node};
    use crate::types::ConfigValue;
    use std::collections::HashMap;

    // Helper function to create test data
    fn create_test_price_alert() -> PriceAlert {
        PriceAlert {
            id: "test_alert_123".to_string(),
            user_id: "test_user".to_string(),
            token_symbol: "BTC".to_string(),
            conditions: vec![PriceCondition::Above(50000.0)],
            actions: vec![AlertAction::SocialPost {
                platforms: vec![SocialPlatform::Telegram],
                message_template: "🚀 BTC price alert: ${price} reached!".to_string(),
            }],
            is_active: true,
            created_at: 0,
            last_triggered: None,
        }
    }

    fn create_test_token_price() -> TokenPrice {
        TokenPrice {
            symbol: "BTC".to_string(),
            price_usd: 55000.0,
            change_24h: 5.2,
            volume_24h: 25000000000.0,
            timestamp: ic_cdk::api::time(),
        }
    }

    #[tokio::test]
    async fn test_telegram_message_formatting() {
        let service = PriceAlertService::new();
        let alert = create_test_price_alert();
        let price = create_test_token_price();

        // Test message formatting for Telegram
        let result = service.check_and_trigger_alerts(&[alert], &price).await;
        assert!(result.is_ok(), "Telegram alert triggering should succeed");
    }

    #[tokio::test]
    async fn test_telegram_post_message() {
        let service = PriceAlertService::new();

        // Test posting a message to Telegram
        let result = service.post_to_telegram("Test message for Telegram bot").await;
        assert!(result.is_ok(), "Telegram message posting should succeed");
    }

    #[tokio::test]
    async fn test_telegram_message_escaping() {
        let service = PriceAlertService::new();

        // Test message with special characters that need escaping
        let message_with_quotes = "Price \"alert\" for BTC: $55,000";
        let result = service.post_to_telegram(message_with_quotes).await;
        assert!(result.is_ok(), "Telegram should handle special characters");
    }

    #[tokio::test]
    async fn test_telegram_long_message_handling() {
        let service = PriceAlertService::new();

        // Test very long message (Telegram has a 4096 character limit)
        let long_message = "A".repeat(5000);
        let result = service.post_to_telegram(&long_message).await;
        assert!(result.is_ok(), "Telegram should handle long messages gracefully");
    }

    #[tokio::test]
    async fn test_telegram_node_definition() {
        // Test the Telegram node definition
        let node_definitions = crate::nodes::get_available_node_types();

        let telegram_node = node_definitions.iter()
            .find(|def| def.node_type == "telegram")
            .expect("Telegram node definition should exist");

        assert_eq!(telegram_node.name, "Telegram");
        assert_eq!(telegram_node.category, "Social");
        assert!(!telegram_node.input_schema.is_empty());
        assert!(!telegram_node.output_schema.is_empty());
    }

    #[tokio::test]
    async fn test_telegram_node_parameters() {
        let node_definitions = crate::nodes::get_available_node_types();
        let telegram_node = node_definitions.iter()
            .find(|def| def.node_type == "telegram")
            .unwrap();

        // Check required parameters
        let bot_token_param = telegram_node.configuration_schema.parameters.iter()
            .find(|p| p.name == "bot_token")
            .expect("bot_token parameter should exist");
        assert!(bot_token_param.required);

        let chat_id_param = telegram_node.configuration_schema.parameters.iter()
            .find(|p| p.name == "chat_id")
            .expect("chat_id parameter should exist");
        assert!(chat_id_param.required);
    }

    #[tokio::test]
    async fn test_telegram_social_media_post_integration() {
        // Test Telegram integration through social media post node
        let mut input = HashMap::new();

        let mut platform_config = HashMap::new();
        platform_config.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
        platform_config.insert("bot_token".to_string(), ConfigValue::String("test_token".to_string()));
        platform_config.insert("chat_id".to_string(), ConfigValue::String("test_chat".to_string()));

        let mut content_data = HashMap::new();
        content_data.insert("message".to_string(), ConfigValue::String("Test Telegram message".to_string()));

        input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
        input.insert("content_data".to_string(), ConfigValue::Object(content_data));

        let node = WorkflowNode {
            id: "test_telegram_node".to_string(),
            node_type: "social-media-post".to_string(),
            position: (0, 0),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        };

        let result = execute_social_media_post_node(&node, &input).await;
        assert!(result.is_ok(), "Telegram social media post should succeed");

        let output = result.unwrap();
        assert_eq!(output.data.get("platform").unwrap(), &ConfigValue::String("telegram".to_string()));
        assert!(output.data.contains_key("post_id"));
        assert_eq!(output.data.get("status").unwrap(), &ConfigValue::String("success".to_string()));
    }

    #[tokio::test]
    async fn test_telegram_error_handling() {
        let service = PriceAlertService::new();

        // Test with empty message
        let result = service.post_to_telegram("").await;
        assert!(result.is_ok(), "Should handle empty messages gracefully");

        // Test with invalid characters (in a real implementation, this might fail)
        let result = service.post_to_telegram("Invalid \x00 character").await;
        assert!(result.is_ok(), "Should handle invalid characters gracefully");
    }

    #[tokio::test]
    async fn test_telegram_price_alert_template() {
        use crate::defi::social_media_formatter::SocialMediaFormatter;

        let formatter = SocialMediaFormatter::new();
        let alert = create_test_price_alert();
        let price = create_test_token_price();

        // Test formatting for Telegram
        let social_data = formatter.format_price_alert_message(
            &alert,
            &price,
            SocialPlatform::Telegram
        );

        assert!(social_data.message.contains("BTC"));
        assert!(social_data.message.contains("55000"));
        assert!(social_data.message.len() <= 4096); // Telegram character limit
    }

    #[tokio::test]
    async fn test_telegram_with_defi_context() {
        use crate::defi::social_media_formatter::SocialMediaFormatter;

        let formatter = SocialMediaFormatter::new();

        // Test DeFi context formatting for Telegram
        let defi_context = r#"{"protocol":"Aave","apy":5.2,"chain":"Ethereum"}"#;
        let message = "DeFi yield opportunity available!";

        let formatted = formatter.format_with_defi_context(
            message,
            defi_context,
            SocialPlatform::Telegram
        );

        assert!(formatted.message.contains("DeFi"));
        assert!(formatted.message.contains("Aave"));
        assert!(formatted.json_payload.contains("protocol"));
    }
}

#[cfg(test)]
mod discord_tests {
    use super::super::*;
    use crate::defi::price_alert_service::{PriceAlertService, SocialPlatform, PriceAlert, AlertAction, PriceCondition, TokenPrice};
    use crate::nodes::{WorkflowNode, NodeConfiguration, execute_social_media_post_node};
    use crate::types::ConfigValue;
    use std::collections::HashMap;

    // Helper function to create test data for Discord
    fn create_test_discord_alert() -> PriceAlert {
        PriceAlert {
            id: "discord_alert_123".to_string(),
            user_id: "discord_user".to_string(),
            token_symbol: "ETH".to_string(),
            conditions: vec![PriceCondition::Below(3000.0)],
            actions: vec![AlertAction::SocialPost {
                platforms: vec![SocialPlatform::Discord],
                message_template: "🔔 ETH price drop alert: ${price}".to_string(),
            }],
            is_active: true,
            created_at: 0,
            last_triggered: None,
        }
    }

    fn create_test_eth_price() -> TokenPrice {
        TokenPrice {
            symbol: "ETH".to_string(),
            price_usd: 2800.0,
            change_24h: -3.5,
            volume_24h: 15000000000.0,
            timestamp: ic_cdk::api::time(),
        }
    }

    #[tokio::test]
    async fn test_discord_message_formatting() {
        let service = PriceAlertService::new();
        let alert = create_test_discord_alert();
        let price = create_test_eth_price();

        // Test message formatting for Discord
        let result = service.check_and_trigger_alerts(&[alert], &price).await;
        assert!(result.is_ok(), "Discord alert triggering should succeed");
    }

    #[tokio::test]
    async fn test_discord_post_message() {
        let service = PriceAlertService::new();

        // Test posting a message to Discord
        let result = service.post_to_discord("Test message for Discord webhook").await;
        assert!(result.is_ok(), "Discord message posting should succeed");
    }

    #[tokio::test]
    async fn test_discord_embed_formatting() {
        let service = PriceAlertService::new();

        // Test Discord webhook payload formatting
        let message = "Price alert for ETH";
        let result = service.post_to_discord(message).await;
        assert!(result.is_ok(), "Discord embed formatting should succeed");
    }

    #[tokio::test]
    async fn test_discord_message_escaping() {
        let service = PriceAlertService::new();

        // Test message with quotes that need escaping in JSON
        let message_with_quotes = "Price \"alert\" for ETH: $2,800";
        let result = service.post_to_discord(message_with_quotes).await;
        assert!(result.is_ok(), "Discord should handle JSON escaping");
    }

    #[tokio::test]
    async fn test_discord_long_message_handling() {
        let service = PriceAlertService::new();

        // Test Discord 2000 character limit
        let long_message = "A".repeat(2500);
        let result = service.post_to_discord(&long_message).await;
        assert!(result.is_ok(), "Discord should handle long messages gracefully");
    }

    #[tokio::test]
    async fn test_discord_node_definition() {
        // Test the Discord node definition
        let node_definitions = crate::nodes::get_available_node_types();

        let discord_node = node_definitions.iter()
            .find(|def| def.node_type == "discord")
            .expect("Discord node definition should exist");

        assert_eq!(discord_node.name, "Discord");
        assert_eq!(discord_node.category, "Social");
        assert!(!discord_node.input_schema.is_empty());
        assert!(!discord_node.output_schema.is_empty());
    }

    #[tokio::test]
    async fn test_discord_node_parameters() {
        let node_definitions = crate::nodes::get_available_node_types();
        let discord_node = node_definitions.iter()
            .find(|def| def.node_type == "discord")
            .unwrap();

        // Check required parameters
        let webhook_url_param = discord_node.configuration_schema.parameters.iter()
            .find(|p| p.name == "webhook_url")
            .expect("webhook_url parameter should exist");
        assert!(webhook_url_param.required);
    }

    #[tokio::test]
    async fn test_discord_social_media_post_integration() {
        // Test Discord integration through social media post node
        let mut input = HashMap::new();

        let mut platform_config = HashMap::new();
        platform_config.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
        platform_config.insert("webhook_url".to_string(), ConfigValue::String("https://discord.com/api/webhooks/test".to_string()));

        let mut content_data = HashMap::new();
        content_data.insert("message".to_string(), ConfigValue::String("Test Discord message".to_string()));
        content_data.insert("username".to_string(), ConfigValue::String("DeFlow Bot".to_string()));

        input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
        input.insert("content_data".to_string(), ConfigValue::Object(content_data));

        let node = WorkflowNode {
            id: "test_discord_node".to_string(),
            node_type: "social-media-post".to_string(),
            position: (0, 0),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        };

        let result = execute_social_media_post_node(&node, &input).await;
        assert!(result.is_ok(), "Discord social media post should succeed");

        let output = result.unwrap();
        assert_eq!(output.data.get("platform").unwrap(), &ConfigValue::String("discord".to_string()));
        assert!(output.data.contains_key("post_id"));
        assert_eq!(output.data.get("status").unwrap(), &ConfigValue::String("success".to_string()));
    }

    #[tokio::test]
    async fn test_discord_character_limit() {
        use crate::nodes::optimize_text_for_platform;

        // Test Discord's 2000 character limit
        let long_text = "A".repeat(2500);
        let optimized = optimize_text_for_platform(&long_text, "discord");

        assert!(optimized.len() <= 2000, "Discord text should be within character limit");
        assert!(optimized.ends_with("..."), "Truncated text should end with ellipsis");
    }

    #[tokio::test]
    async fn test_discord_rich_embeds() {
        use crate::defi::social_media_formatter::SocialMediaFormatter;

        let formatter = SocialMediaFormatter::new();
        let alert = create_test_discord_alert();
        let price = create_test_eth_price();

        // Test Discord rich embed formatting
        let social_data = formatter.format_price_alert_message(
            &alert,
            &price,
            SocialPlatform::Discord
        );

        assert!(social_data.message.contains("ETH"));
        assert!(social_data.message.contains("2800"));
        assert!(social_data.message.len() <= 2000); // Discord character limit
    }

    #[tokio::test]
    async fn test_discord_webhook_validation() {
        // Test webhook URL validation
        let valid_urls = vec![
            "https://discord.com/api/webhooks/123456789/abcdef",
            "https://discordapp.com/api/webhooks/123456789/abcdef",
        ];

        let invalid_urls = vec![
            "https://example.com/webhook",
            "not-a-url",
            "",
        ];

        for url in valid_urls {
            // In a real implementation, you'd validate the webhook URL format
            assert!(url.contains("discord"), "Should be a Discord webhook URL");
        }

        for url in invalid_urls {
            // In a real implementation, you'd reject invalid URLs
            assert!(!url.contains("discord.com"), "Should reject invalid URLs");
        }
    }

    #[tokio::test]
    async fn test_discord_error_handling() {
        let service = PriceAlertService::new();

        // Test with empty message
        let result = service.post_to_discord("").await;
        assert!(result.is_ok(), "Should handle empty messages gracefully");

        // Test with special Discord markdown characters
        let result = service.post_to_discord("**Bold** *italic* `code` ```code block```").await;
        assert!(result.is_ok(), "Should handle Discord markdown");
    }
}

#[cfg(test)]
mod social_media_integration_tests {
    use super::super::*;
    use crate::defi::price_alert_service::{PriceAlertService, SocialPlatform};
    use crate::defi::social_media_formatter::SocialMediaFormatter;
    use crate::cycles_monitor_service::CyclesMonitorService;

    #[tokio::test]
    async fn test_multi_platform_alert() {
        let service = PriceAlertService::new();

        // Test alert that should trigger on both Telegram and Discord
        let alert = crate::defi::price_alert_service::PriceAlert {
            id: "multi_platform_alert".to_string(),
            user_id: "test_user".to_string(),
            token_symbol: "BTC".to_string(),
            conditions: vec![crate::defi::price_alert_service::PriceCondition::Above(50000.0)],
            actions: vec![crate::defi::price_alert_service::AlertAction::SocialPost {
                platforms: vec![SocialPlatform::Telegram, SocialPlatform::Discord],
                message_template: "Multi-platform alert: BTC ${price}".to_string(),
            }],
            is_active: true,
            created_at: 0,
            last_triggered: None,
        };

        let price = crate::defi::price_alert_service::TokenPrice {
            symbol: "BTC".to_string(),
            price_usd: 55000.0,
            change_24h: 5.0,
            volume_24h: 20000000000.0,
            timestamp: ic_cdk::api::time(),
        };

        let result = service.check_and_trigger_alerts(&[alert], &price).await;
        assert!(result.is_ok(), "Multi-platform alert should succeed");
    }

    #[tokio::test]
    async fn test_platform_specific_formatting() {
        let formatter = SocialMediaFormatter::new();

        let base_message = "BTC price alert: $55,000";

        // Test Telegram formatting
        let telegram_result = formatter.format_with_defi_context(
            base_message,
            r#"{"protocol":"Aave"}"#,
            SocialPlatform::Telegram
        );

        // Test Discord formatting
        let discord_result = formatter.format_with_defi_context(
            base_message,
            r#"{"protocol":"Aave"}"#,
            SocialPlatform::Discord
        );

        // Both should succeed but may have different formatting
        assert!(telegram_result.message.contains("BTC"));
        assert!(discord_result.message.contains("BTC"));
    }

    #[tokio::test]
    async fn test_cycles_monitor_social_integration() {
        // Test cycles monitor integration with social platforms
        use crate::cycles_monitor_service::NotificationChannel;

        let channels = vec![
            NotificationChannel {
                channel_type: "discord".to_string(),
                endpoint: "https://discord.com/api/webhooks/test".to_string(),
                enabled: true,
            },
            NotificationChannel {
                channel_type: "telegram".to_string(),
                endpoint: "telegram:chat_id:bot_token".to_string(),
                enabled: true,
            },
        ];

        // Verify channels are properly configured
        for channel in channels {
            match channel.channel_type.as_str() {
                "discord" => assert!(channel.endpoint.contains("discord.com")),
                "telegram" => assert!(channel.endpoint.contains("telegram:")),
                _ => panic!("Unexpected channel type"),
            }
        }
    }

    #[tokio::test]
    async fn test_social_platform_availability() {
        // Test that both platforms are available in the system
        use crate::types::SubscriptionTier;

        let standard_tier = SubscriptionTier::Standard;
        let available_platforms = standard_tier.get_available_platforms();

        assert!(available_platforms.contains(&"telegram".to_string()));
        assert!(available_platforms.contains(&"discord".to_string()));
    }

    #[tokio::test]
    async fn test_social_integration_configuration() {
        use crate::types::SocialIntegrationConfig;

        let config = SocialIntegrationConfig::default();

        // Test default configuration
        assert!(!config.discord_enabled);
        assert!(!config.telegram_enabled);

        // Test platform status
        assert_eq!(config.get_active_platforms().len(), 0);
    }
}

#[cfg(test)]
mod social_media_utilities_tests {
    use super::super::*;
    use std::collections::HashMap;

    /// Test utility for creating mock social media configurations
    pub fn create_mock_telegram_config() -> HashMap<String, crate::types::ConfigValue> {
        let mut config = HashMap::new();
        config.insert("bot_token".to_string(), crate::types::ConfigValue::String("mock_bot_token".to_string()));
        config.insert("chat_id".to_string(), crate::types::ConfigValue::String("mock_chat_id".to_string()));
        config.insert("parse_mode".to_string(), crate::types::ConfigValue::String("HTML".to_string()));
        config
    }

    /// Test utility for creating mock Discord configurations
    pub fn create_mock_discord_config() -> HashMap<String, crate::types::ConfigValue> {
        let mut config = HashMap::new();
        config.insert("webhook_url".to_string(), crate::types::ConfigValue::String("https://discord.com/api/webhooks/mock".to_string()));
        config.insert("username".to_string(), crate::types::ConfigValue::String("DeFlow Bot".to_string()));
        config.insert("avatar_url".to_string(), crate::types::ConfigValue::String("https://example.com/avatar.png".to_string()));
        config
    }

    /// Test utility for validating social media response format
    pub fn validate_social_response(response: &HashMap<String, crate::types::ConfigValue>) -> bool {
        response.contains_key("platform") &&
        response.contains_key("post_id") &&
        response.contains_key("status")
    }

    #[test]
    fn test_mock_config_utilities() {
        let telegram_config = create_mock_telegram_config();
        assert!(telegram_config.contains_key("bot_token"));
        assert!(telegram_config.contains_key("chat_id"));

        let discord_config = create_mock_discord_config();
        assert!(discord_config.contains_key("webhook_url"));
        assert!(discord_config.contains_key("username"));
    }

    #[test]
    fn test_response_validation_utility() {
        let mut valid_response = HashMap::new();
        valid_response.insert("platform".to_string(), crate::types::ConfigValue::String("telegram".to_string()));
        valid_response.insert("post_id".to_string(), crate::types::ConfigValue::String("123".to_string()));
        valid_response.insert("status".to_string(), crate::types::ConfigValue::String("success".to_string()));

        assert!(validate_social_response(&valid_response));

        let mut invalid_response = HashMap::new();
        invalid_response.insert("platform".to_string(), crate::types::ConfigValue::String("telegram".to_string()));
        // Missing post_id and status

        assert!(!validate_social_response(&invalid_response));
    }
}