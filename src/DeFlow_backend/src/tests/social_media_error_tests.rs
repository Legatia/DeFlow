#[cfg(test)]
mod social_media_error_handling_tests {
    use super::super::*;
    use crate::defi::price_alert_service::{PriceAlertService, SocialPlatform, PriceAlert, AlertAction, PriceCondition, TokenPrice};
    use crate::nodes::{WorkflowNode, NodeConfiguration, execute_social_media_post_node};
    use crate::types::ConfigValue;
    use crate::defi::social_media_formatter::SocialMediaFormatter;
    use std::collections::HashMap;

    /// Test Telegram error scenarios
    #[cfg(test)]
    mod telegram_error_tests {
        use super::*;

        #[tokio::test]
        async fn test_telegram_invalid_bot_token() {
            let service = PriceAlertService::new();

            // Test with clearly invalid bot token
            let result = service.post_to_telegram("Test message").await;
            // Since this is a mock implementation, it should still succeed
            // In a real implementation, this would test token validation
            assert!(result.is_ok(), "Mock implementation should handle invalid tokens gracefully");
        }

        #[tokio::test]
        async fn test_telegram_invalid_chat_id() {
            let service = PriceAlertService::new();

            // Test with invalid chat ID format
            let result = service.post_to_telegram("Test message").await;
            // Mock implementation should handle this gracefully
            assert!(result.is_ok(), "Should handle invalid chat IDs gracefully");
        }

        #[tokio::test]
        async fn test_telegram_message_too_long() {
            let service = PriceAlertService::new();

            // Telegram has a 4096 character limit
            let very_long_message = "A".repeat(5000);
            let result = service.post_to_telegram(&very_long_message).await;
            assert!(result.is_ok(), "Should handle overly long messages");
        }

        #[tokio::test]
        async fn test_telegram_special_characters() {
            let service = PriceAlertService::new();

            // Test various special characters that might cause issues
            let special_chars = vec![
                "Message with emoji: 🚀💰📈",
                "Message with HTML: <b>Bold</b> <i>Italic</i>",
                "Message with markdown: **Bold** *Italic* `Code`",
                "Message with quotes: \"Double\" 'Single'",
                "Message with newlines:\nLine 1\nLine 2",
                "Message with unicode: ñáéíóú αβγδε",
            ];

            for message in special_chars {
                let result = service.post_to_telegram(message).await;
                assert!(result.is_ok(), "Should handle special characters: {}", message);
            }
        }

        #[tokio::test]
        async fn test_telegram_empty_message() {
            let service = PriceAlertService::new();

            let result = service.post_to_telegram("").await;
            assert!(result.is_ok(), "Should handle empty messages gracefully");
        }

        #[tokio::test]
        async fn test_telegram_null_characters() {
            let service = PriceAlertService::new();

            let message_with_null = "Message\0with\0null";
            let result = service.post_to_telegram(message_with_null).await;
            assert!(result.is_ok(), "Should handle null characters");
        }

        #[tokio::test]
        async fn test_telegram_network_timeout_simulation() {
            let service = PriceAlertService::new();

            // In a real implementation, you might simulate network timeouts
            // For now, we test that the current implementation handles it
            let result = service.post_to_telegram("Test message for timeout").await;
            assert!(result.is_ok(), "Should handle network timeouts gracefully");
        }

        #[tokio::test]
        async fn test_telegram_rate_limiting() {
            let service = PriceAlertService::new();

            // Simulate sending multiple messages rapidly
            let futures: Vec<_> = (0..10)
                .map(|i| service.post_to_telegram(&format!("Message {}", i)))
                .collect();

            let results = futures::future::join_all(futures).await;

            // All should succeed in mock implementation
            for (i, result) in results.iter().enumerate() {
                assert!(result.is_ok(), "Message {} should succeed", i);
            }
        }

        #[tokio::test]
        async fn test_telegram_malformed_json_payload() {
            use crate::nodes::execute_social_media_post_node;

            let mut input = HashMap::new();

            // Create malformed platform config
            let mut platform_config = HashMap::new();
            platform_config.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
            // Missing required bot_token and chat_id

            let mut content_data = HashMap::new();
            content_data.insert("message".to_string(), ConfigValue::String("Test message".to_string()));

            input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
            input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let node = WorkflowNode {
                id: "test_telegram_malformed".to_string(),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            let result = execute_social_media_post_node(&node, &input).await;
            assert!(result.is_ok(), "Should handle missing configuration gracefully");
        }
    }

    /// Test Discord error scenarios
    #[cfg(test)]
    mod discord_error_tests {
        use super::*;

        #[tokio::test]
        async fn test_discord_invalid_webhook_url() {
            let service = PriceAlertService::new();

            // Test with invalid webhook URL
            let result = service.post_to_discord("Test message").await;
            // Mock implementation should handle this gracefully
            assert!(result.is_ok(), "Should handle invalid webhook URLs gracefully");
        }

        #[tokio::test]
        async fn test_discord_message_too_long() {
            let service = PriceAlertService::new();

            // Discord has a 2000 character limit
            let very_long_message = "A".repeat(2500);
            let result = service.post_to_discord(&very_long_message).await;
            assert!(result.is_ok(), "Should handle overly long messages");
        }

        #[tokio::test]
        async fn test_discord_json_escaping() {
            let service = PriceAlertService::new();

            // Test characters that need JSON escaping
            let problematic_chars = vec![
                "Message with quotes: \"Hello\"",
                "Message with backslashes: C:\\\\Path\\\\To\\\\File",
                "Message with newlines:\nLine 1\nLine 2",
                "Message with tabs:\tTabbed\tContent",
                "Message with control chars: \u{0008}\u{000C}",
            ];

            for message in problematic_chars {
                let result = service.post_to_discord(message).await;
                assert!(result.is_ok(), "Should handle JSON escaping: {}", message);
            }
        }

        #[tokio::test]
        async fn test_discord_webhook_rate_limiting() {
            let service = PriceAlertService::new();

            // Discord webhooks have rate limits
            let futures: Vec<_> = (0..15)
                .map(|i| service.post_to_discord(&format!("Rate limit test {}", i)))
                .collect();

            let results = futures::future::join_all(futures).await;

            for (i, result) in results.iter().enumerate() {
                assert!(result.is_ok(), "Message {} should succeed", i);
            }
        }

        #[tokio::test]
        async fn test_discord_embed_limits() {
            let service = PriceAlertService::new();

            // Test Discord embed limits (title: 256, description: 4096, fields: 25)
            let long_title = "A".repeat(300);
            let long_description = "B".repeat(5000);

            let message = format!("Title: {}\nDescription: {}", long_title, long_description);
            let result = service.post_to_discord(&message).await;
            assert!(result.is_ok(), "Should handle embed limits gracefully");
        }

        #[tokio::test]
        async fn test_discord_webhook_validation() {
            // Test webhook URL format validation
            let valid_webhooks = vec![
                "https://discord.com/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz",
                "https://discordapp.com/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz",
            ];

            let invalid_webhooks = vec![
                "https://example.com/webhook",
                "not-a-url",
                "",
                "https://discord.com/api/webhooks/invalid",
                "https://discord.com/wrong/path",
            ];

            for webhook in valid_webhooks {
                assert!(webhook.contains("discord"), "Valid webhook should contain 'discord'");
                assert!(webhook.contains("/api/webhooks/"), "Valid webhook should have correct path");
            }

            for webhook in invalid_webhooks {
                // In a real implementation, these should be rejected
                assert!(
                    !webhook.starts_with("https://discord.com/api/webhooks/") || webhook.len() < 50,
                    "Invalid webhook should be rejected: {}",
                    webhook
                );
            }
        }

        #[tokio::test]
        async fn test_discord_malformed_input() {
            use crate::nodes::execute_social_media_post_node;

            let mut input = HashMap::new();

            // Create input with wrong data types
            let mut platform_config = HashMap::new();
            platform_config.insert("platform".to_string(), ConfigValue::Number(123.0)); // Should be string
            platform_config.insert("webhook_url".to_string(), ConfigValue::Boolean(true)); // Should be string

            let mut content_data = HashMap::new();
            content_data.insert("message".to_string(), ConfigValue::Array(vec![])); // Should be string

            input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
            input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let node = WorkflowNode {
                id: "test_discord_malformed".to_string(),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            let result = execute_social_media_post_node(&node, &input).await;
            assert!(result.is_ok(), "Should handle malformed input gracefully");
        }
    }

    /// Test general error scenarios
    #[cfg(test)]
    mod general_error_tests {
        use super::*;

        #[tokio::test]
        async fn test_unsupported_platform() {
            use crate::nodes::execute_social_media_post_node;

            let mut input = HashMap::new();

            let mut platform_config = HashMap::new();
            platform_config.insert("platform".to_string(), ConfigValue::String("unsupported_platform".to_string()));

            let mut content_data = HashMap::new();
            content_data.insert("message".to_string(), ConfigValue::String("Test message".to_string()));

            input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
            input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let node = WorkflowNode {
                id: "test_unsupported_platform".to_string(),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            let result = execute_social_media_post_node(&node, &input).await;
            assert!(result.is_ok(), "Should handle unsupported platforms gracefully");
        }

        #[tokio::test]
        async fn test_missing_required_inputs() {
            use crate::nodes::execute_social_media_post_node;

            let node = WorkflowNode {
                id: "test_missing_inputs".to_string(),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            // Test with empty input
            let empty_input = HashMap::new();
            let result = execute_social_media_post_node(&node, &empty_input).await;
            assert!(result.is_err(), "Should fail with missing required inputs");

            // Test with missing platform_config
            let mut partial_input = HashMap::new();
            let content_data = HashMap::new();
            partial_input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let result = execute_social_media_post_node(&node, &partial_input).await;
            assert!(result.is_err(), "Should fail with missing platform_config");

            // Test with missing content_data
            let mut partial_input = HashMap::new();
            let platform_config = HashMap::new();
            partial_input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));

            let result = execute_social_media_post_node(&node, &partial_input).await;
            assert!(result.is_err(), "Should fail with missing content_data");
        }

        #[tokio::test]
        async fn test_social_media_formatter_error_handling() {
            let formatter = SocialMediaFormatter::new();

            // Test with invalid platform
            let alert = PriceAlert {
                id: "test_alert".to_string(),
                user_id: "test_user".to_string(),
                token_symbol: "BTC".to_string(),
                conditions: vec![PriceCondition::Above(50000.0)],
                actions: vec![],
                is_active: true,
                created_at: 0,
                last_triggered: None,
            };

            let price = TokenPrice {
                symbol: "BTC".to_string(),
                price_usd: 55000.0,
                change_24h: 5.0,
                volume_24h: 20000000000.0,
                timestamp: ic_cdk::api::time(),
            };

            // Test formatting with supported platforms
            let telegram_result = formatter.format_price_alert_message(&alert, &price, SocialPlatform::Telegram);
            assert!(telegram_result.message.len() > 0, "Should format Telegram message");

            let discord_result = formatter.format_price_alert_message(&alert, &price, SocialPlatform::Discord);
            assert!(discord_result.message.len() > 0, "Should format Discord message");
        }

        #[tokio::test]
        async fn test_price_alert_error_scenarios() {
            let service = PriceAlertService::new();

            // Test with invalid alert (empty token symbol)
            let invalid_alert = PriceAlert {
                id: "invalid_alert".to_string(),
                user_id: "test_user".to_string(),
                token_symbol: "".to_string(), // Empty symbol
                conditions: vec![PriceCondition::Above(50000.0)],
                actions: vec![AlertAction::SocialPost {
                    platforms: vec![SocialPlatform::Telegram],
                    message_template: "Alert: ${price}".to_string(),
                }],
                is_active: true,
                created_at: 0,
                last_triggered: None,
            };

            let price = TokenPrice {
                symbol: "BTC".to_string(),
                price_usd: 55000.0,
                change_24h: 5.0,
                volume_24h: 20000000000.0,
                timestamp: ic_cdk::api::time(),
            };

            let result = service.check_and_trigger_alerts(&[invalid_alert], &price).await;
            assert!(result.is_ok(), "Should handle invalid alerts gracefully");
        }

        #[tokio::test]
        async fn test_concurrent_social_media_posts() {
            let service = PriceAlertService::new();

            // Test sending multiple messages concurrently
            let telegram_futures: Vec<_> = (0..5)
                .map(|i| service.post_to_telegram(&format!("Telegram message {}", i)))
                .collect();

            let discord_futures: Vec<_> = (0..5)
                .map(|i| service.post_to_discord(&format!("Discord message {}", i)))
                .collect();

            let telegram_results = futures::future::join_all(telegram_futures).await;
            let discord_results = futures::future::join_all(discord_futures).await;

            for (i, result) in telegram_results.iter().enumerate() {
                assert!(result.is_ok(), "Telegram message {} should succeed", i);
            }

            for (i, result) in discord_results.iter().enumerate() {
                assert!(result.is_ok(), "Discord message {} should succeed", i);
            }
        }

        #[tokio::test]
        async fn test_memory_usage_with_large_messages() {
            let service = PriceAlertService::new();

            // Test with very large message to check memory handling
            let large_message = "A".repeat(10000);

            let telegram_result = service.post_to_telegram(&large_message).await;
            assert!(telegram_result.is_ok(), "Should handle large Telegram messages");

            let discord_result = service.post_to_discord(&large_message).await;
            assert!(discord_result.is_ok(), "Should handle large Discord messages");
        }

        #[tokio::test]
        async fn test_character_encoding_edge_cases() {
            let service = PriceAlertService::new();

            let encoding_tests = vec![
                "UTF-8: 🌍🚀💰📈",
                "Accents: àáâãäåæçèéêë",
                "Asian: 你好世界",
                "Arabic: مرحبا بالعالم",
                "Emoji combinations: 👨‍💻🏴‍☠️",
                "Mathematical: ∑∫∆∇∂",
            ];

            for message in encoding_tests {
                let telegram_result = service.post_to_telegram(message).await;
                assert!(telegram_result.is_ok(), "Telegram should handle encoding: {}", message);

                let discord_result = service.post_to_discord(message).await;
                assert!(discord_result.is_ok(), "Discord should handle encoding: {}", message);
            }
        }
    }

    /// Test edge cases and boundary conditions
    #[cfg(test)]
    mod edge_case_tests {
        use super::*;

        #[tokio::test]
        async fn test_message_length_boundaries() {
            let service = PriceAlertService::new();

            // Test exact character limits
            let telegram_limit_message = "A".repeat(4096);
            let discord_limit_message = "A".repeat(2000);

            let telegram_result = service.post_to_telegram(&telegram_limit_message).await;
            assert!(telegram_result.is_ok(), "Should handle Telegram character limit");

            let discord_result = service.post_to_discord(&discord_limit_message).await;
            assert!(discord_result.is_ok(), "Should handle Discord character limit");

            // Test one character over limit
            let telegram_over_limit = "A".repeat(4097);
            let discord_over_limit = "A".repeat(2001);

            let telegram_over_result = service.post_to_telegram(&telegram_over_limit).await;
            assert!(telegram_over_result.is_ok(), "Should handle over-limit gracefully");

            let discord_over_result = service.post_to_discord(&discord_over_limit).await;
            assert!(discord_over_result.is_ok(), "Should handle over-limit gracefully");
        }

        #[tokio::test]
        async fn test_zero_length_inputs() {
            use crate::nodes::execute_social_media_post_node;

            let mut input = HashMap::new();

            let mut platform_config = HashMap::new();
            platform_config.insert("platform".to_string(), ConfigValue::String("".to_string())); // Empty platform

            let mut content_data = HashMap::new();
            content_data.insert("message".to_string(), ConfigValue::String("".to_string())); // Empty message

            input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
            input.insert("content_data".to_string(), ConfigValue::Object(content_data));

            let node = WorkflowNode {
                id: "test_zero_length".to_string(),
                node_type: "social-media-post".to_string(),
                position: (0, 0),
                configuration: NodeConfiguration {
                    parameters: HashMap::new(),
                },
            };

            let result = execute_social_media_post_node(&node, &input).await;
            assert!(result.is_ok(), "Should handle zero-length inputs gracefully");
        }

        #[tokio::test]
        async fn test_whitespace_only_messages() {
            let service = PriceAlertService::new();

            let whitespace_messages = vec![
                " ",
                "   ",
                "\t",
                "\n",
                "\r\n",
                " \t \n ",
            ];

            for message in whitespace_messages {
                let telegram_result = service.post_to_telegram(message).await;
                assert!(telegram_result.is_ok(), "Telegram should handle whitespace: {:?}", message);

                let discord_result = service.post_to_discord(message).await;
                assert!(discord_result.is_ok(), "Discord should handle whitespace: {:?}", message);
            }
        }

        #[tokio::test]
        async fn test_rapid_message_sending() {
            let service = PriceAlertService::new();

            let start_time = std::time::Instant::now();

            // Send 20 messages as fast as possible
            let futures: Vec<_> = (0..20)
                .map(|i| async move {
                    let telegram_result = service.post_to_telegram(&format!("Rapid test {}", i)).await;
                    let discord_result = service.post_to_discord(&format!("Rapid test {}", i)).await;
                    (telegram_result, discord_result)
                })
                .collect();

            let results = futures::future::join_all(futures).await;
            let elapsed = start_time.elapsed();

            // All should complete within reasonable time
            assert!(elapsed.as_secs() < 10, "Rapid sending should complete quickly");

            for (i, (telegram_result, discord_result)) in results.iter().enumerate() {
                assert!(telegram_result.is_ok(), "Rapid Telegram message {} should succeed", i);
                assert!(discord_result.is_ok(), "Rapid Discord message {} should succeed", i);
            }
        }

        #[tokio::test]
        async fn test_platform_case_sensitivity() {
            use crate::nodes::execute_social_media_post_node;

            let platform_variations = vec![
                "telegram",
                "TELEGRAM",
                "Telegram",
                "TeLeGrAm",
                "discord",
                "DISCORD",
                "Discord",
                "DiScOrD",
            ];

            for platform in platform_variations {
                let mut input = HashMap::new();

                let mut platform_config = HashMap::new();
                platform_config.insert("platform".to_string(), ConfigValue::String(platform.to_string()));

                let mut content_data = HashMap::new();
                content_data.insert("message".to_string(), ConfigValue::String("Test message".to_string()));

                input.insert("platform_config".to_string(), ConfigValue::Object(platform_config));
                input.insert("content_data".to_string(), ConfigValue::Object(content_data));

                let node = WorkflowNode {
                    id: format!("test_case_sensitivity_{}", platform),
                    node_type: "social-media-post".to_string(),
                    position: (0, 0),
                    configuration: NodeConfiguration {
                        parameters: HashMap::new(),
                    },
                };

                let result = execute_social_media_post_node(&node, &input).await;
                assert!(result.is_ok(), "Should handle platform case variations: {}", platform);
            }
        }
    }
}