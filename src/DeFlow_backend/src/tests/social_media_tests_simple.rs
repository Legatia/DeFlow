#[cfg(test)]
mod telegram_discord_tests {
    use crate::defi::price_alert_service::{PriceAlertManager, SocialPlatform, PriceAlert, AlertAction, PriceCondition, TokenPrice};
    use crate::nodes::{WorkflowNode, NodeConfiguration, execute_social_media_post_node};
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

    #[test]
    fn test_telegram_node_definition_exists() {
        // Test that Telegram node definition exists
        let node_definitions = crate::nodes::get_available_node_types();

        let telegram_node = node_definitions.iter()
            .find(|def| def.node_type == "telegram");

        assert!(telegram_node.is_some(), "Telegram node definition should exist");

        if let Some(node) = telegram_node {
            assert_eq!(node.name, "Telegram");
            assert_eq!(node.category, "Social");
            assert!(!node.input_schema.is_empty());
            assert!(!node.output_schema.is_empty());
        }
    }

    #[test]
    fn test_discord_node_definition_exists() {
        // Test that Discord node definition exists
        let node_definitions = crate::nodes::get_available_node_types();

        let discord_node = node_definitions.iter()
            .find(|def| def.node_type == "discord");

        assert!(discord_node.is_some(), "Discord node definition should exist");

        if let Some(node) = discord_node {
            assert_eq!(node.name, "Discord");
            assert_eq!(node.category, "Social");
            assert!(!node.input_schema.is_empty());
            assert!(!node.output_schema.is_empty());
        }
    }

    #[tokio::test]
    async fn test_social_media_post_node_telegram() {
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
    async fn test_social_media_post_node_discord() {
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

    #[test]
    fn test_price_alert_structure() {
        let alert = create_test_price_alert();
        assert_eq!(alert.token_symbol, "BTC");
        assert!(alert.is_active);
        assert!(!alert.conditions.is_empty());
        assert!(!alert.actions.is_empty());
    }

    #[test]
    fn test_social_platform_enum() {
        // Test that social platforms are properly defined
        let telegram = SocialPlatform::Telegram;
        let discord = SocialPlatform::Discord;

        // These should compile without issues
        match telegram {
            SocialPlatform::Telegram => assert!(true),
            _ => assert!(false, "Should match Telegram"),
        }

        match discord {
            SocialPlatform::Discord => assert!(true),
            _ => assert!(false, "Should match Discord"),
        }
    }

    #[test]
    fn test_alert_action_social_post() {
        let action = AlertAction::SocialPost {
            platforms: vec![SocialPlatform::Telegram, SocialPlatform::Discord],
            message_template: "Test message: ${price}".to_string(),
        };

        match action {
            AlertAction::SocialPost { platforms, message_template } => {
                assert_eq!(platforms.len(), 2);
                assert!(message_template.contains("${price}"));
            }
            _ => assert!(false, "Should be SocialPost action"),
        }
    }

    #[tokio::test]
    async fn test_missing_platform_config() {
        // Test error handling with missing platform config
        let mut input = HashMap::new();

        let content_data = HashMap::new();
        input.insert("content_data".to_string(), ConfigValue::Object(content_data));
        // Missing platform_config

        let node = WorkflowNode {
            id: "test_missing_config".to_string(),
            node_type: "social-media-post".to_string(),
            position: (0, 0),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        };

        let result = execute_social_media_post_node(&node, &input).await;
        assert!(result.is_err(), "Should fail with missing platform config");
    }

    #[tokio::test]
    async fn test_missing_content_data() {
        // Test error handling with missing content data
        let mut input = HashMap::new();

        let platform_config = HashMap::new();
        input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
        // Missing content_data

        let node = WorkflowNode {
            id: "test_missing_content".to_string(),
            node_type: "social-media-post".to_string(),
            position: (0, 0),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        };

        let result = execute_social_media_post_node(&node, &input).await;
        assert!(result.is_err(), "Should fail with missing content data");
    }

    #[tokio::test]
    async fn test_empty_input() {
        // Test error handling with completely empty input
        let empty_input = HashMap::new();

        let node = WorkflowNode {
            id: "test_empty_input".to_string(),
            node_type: "social-media-post".to_string(),
            position: (0, 0),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        };

        let result = execute_social_media_post_node(&node, &empty_input).await;
        assert!(result.is_err(), "Should fail with empty input");
    }

    #[tokio::test]
    async fn test_platform_switching() {
        // Test switching between platforms
        let platforms = vec!["telegram", "discord"];

        for platform in platforms {
            let mut input = HashMap::new();

            let mut platform_config = HashMap::new();
            platform_config.insert("platform".to_string(), ConfigValue::String(platform.to_string()));

            let mut content_data = HashMap::new();
            content_data.insert("message".to_string(), ConfigValue::String(format!("Test {} message", platform)));

            input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
            input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let node = WorkflowNode {
                id: format!("test_{}_node", platform),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            let result = execute_social_media_post_node(&node, &input).await;
            assert!(result.is_ok(), "Platform {} should succeed", platform);

            if let Ok(output) = result {
                assert_eq!(output.data.get("platform").unwrap(), &ConfigValue::String(platform.to_string()));
            }
        }
    }

    #[test]
    fn test_token_price_structure() {
        let price = create_test_token_price();
        assert_eq!(price.symbol, "BTC");
        assert!(price.price_usd > 0.0);
        assert!(price.volume_24h > 0.0);
        assert!(price.timestamp > 0);
    }

    #[test]
    fn test_price_conditions() {
        // Test different price condition types
        let above_condition = PriceCondition::Above(50000.0);
        let below_condition = PriceCondition::Below(40000.0);

        match above_condition {
            PriceCondition::Above(price) => assert_eq!(price, 50000.0),
            _ => assert!(false, "Should be Above condition"),
        }

        match below_condition {
            PriceCondition::Below(price) => assert_eq!(price, 40000.0),
            _ => assert!(false, "Should be Below condition"),
        }
    }
}

/// Test utilities module for social media integration testing
#[cfg(test)]
pub mod test_utilities {
    use crate::types::ConfigValue;
    use std::collections::HashMap;

    /// Create mock Telegram configuration for testing
    pub fn create_mock_telegram_config() -> HashMap<String, ConfigValue> {
        let mut config = HashMap::new();
        config.insert("bot_token".to_string(), ConfigValue::String("mock_bot_token".to_string()));
        config.insert("chat_id".to_string(), ConfigValue::String("mock_chat_id".to_string()));
        config.insert("parse_mode".to_string(), ConfigValue::String("HTML".to_string()));
        config
    }

    /// Create mock Discord configuration for testing
    pub fn create_mock_discord_config() -> HashMap<String, ConfigValue> {
        let mut config = HashMap::new();
        config.insert("webhook_url".to_string(), ConfigValue::String("https://discord.com/api/webhooks/mock".to_string()));
        config.insert("username".to_string(), ConfigValue::String("DeFlow Bot".to_string()));
        config.insert("avatar_url".to_string(), ConfigValue::String("https://example.com/avatar.png".to_string()));
        config
    }

    /// Validate social media response format
    pub fn validate_social_response(response: &HashMap<String, ConfigValue>) -> bool {
        response.contains_key("platform") &&
        response.contains_key("post_id") &&
        response.contains_key("status")
    }

    /// Helper to create test message content
    pub fn create_test_message_content(message: &str) -> HashMap<String, ConfigValue> {
        let mut content = HashMap::new();
        content.insert("message".to_string(), ConfigValue::String(message.to_string()));
        content
    }

    #[test]
    fn test_mock_utilities() {
        let telegram_config = create_mock_telegram_config();
        assert!(telegram_config.contains_key("bot_token"));
        assert!(telegram_config.contains_key("chat_id"));

        let discord_config = create_mock_discord_config();
        assert!(discord_config.contains_key("webhook_url"));
        assert!(discord_config.contains_key("username"));

        let message_content = create_test_message_content("Test message");
        assert!(message_content.contains_key("message"));
    }

    #[test]
    fn test_response_validation() {
        let mut valid_response = HashMap::new();
        valid_response.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
        valid_response.insert("post_id".to_string(), ConfigValue::String("123".to_string()));
        valid_response.insert("status".to_string(), ConfigValue::String("success".to_string()));

        assert!(validate_social_response(&valid_response));

        let mut invalid_response = HashMap::new();
        invalid_response.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
        // Missing post_id and status

        assert!(!validate_social_response(&invalid_response));
    }
}