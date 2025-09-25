#[cfg(test)]
mod social_workflow_integration_tests {
    use super::super::*;
    use crate::workflow::{create_workflow, WorkflowData};
    use crate::execution::start_execution;
    use crate::nodes::{WorkflowNode, NodeConfiguration};
    use crate::types::{ConfigValue, Workflow};
    use std::collections::HashMap;

    /// Test creating a complete workflow with Telegram notifications
    #[tokio::test]
    async fn test_telegram_workflow_creation() {
        let workflow_data = create_test_telegram_workflow();

        let result = create_workflow(workflow_data).await;
        assert!(result.is_ok(), "Should create Telegram workflow successfully");

        let workflow_id = result.unwrap();
        assert!(!workflow_id.is_empty(), "Should return valid workflow ID");
    }

    /// Test creating a complete workflow with Discord notifications
    #[tokio::test]
    async fn test_discord_workflow_creation() {
        let workflow_data = create_test_discord_workflow();

        let result = create_workflow(workflow_data).await;
        assert!(result.is_ok(), "Should create Discord workflow successfully");

        let workflow_id = result.unwrap();
        assert!(!workflow_id.is_empty(), "Should return valid workflow ID");
    }

    /// Test workflow execution with Telegram node
    #[tokio::test]
    async fn test_telegram_workflow_execution() {
        let workflow_data = create_test_telegram_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Workflow execution should succeed");

        let execution_id = execution_result.unwrap();
        assert!(!execution_id.is_empty(), "Should return valid execution ID");
    }

    /// Test workflow execution with Discord node
    #[tokio::test]
    async fn test_discord_workflow_execution() {
        let workflow_data = create_test_discord_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Workflow execution should succeed");

        let execution_id = execution_result.unwrap();
        assert!(!execution_id.is_empty(), "Should return valid execution ID");
    }

    /// Test multi-platform social media workflow
    #[tokio::test]
    async fn test_multi_platform_social_workflow() {
        let workflow_data = create_test_multi_platform_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Multi-platform workflow should succeed");
    }

    /// Test price alert workflow with social media notifications
    #[tokio::test]
    async fn test_price_alert_social_workflow() {
        let workflow_data = create_test_price_alert_social_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Price alert social workflow should succeed");
    }

    /// Test DeFi yield notification workflow
    #[tokio::test]
    async fn test_defi_yield_notification_workflow() {
        let workflow_data = create_test_defi_yield_notification_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "DeFi yield notification workflow should succeed");
    }

    /// Test workflow with conditional social media posting
    #[tokio::test]
    async fn test_conditional_social_posting_workflow() {
        let workflow_data = create_test_conditional_social_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Conditional social posting should succeed");
    }

    /// Test workflow failure handling with social media error recovery
    #[tokio::test]
    async fn test_social_workflow_error_recovery() {
        let workflow_data = create_test_error_recovery_workflow();
        let workflow_id = create_workflow(workflow_data).await.unwrap();

        // Execute the workflow (should handle errors gracefully)
        let execution_result = start_execution(workflow_id, None).await;
        assert!(execution_result.is_ok(), "Error recovery workflow should handle failures");
    }

    // Helper functions to create test workflows

    fn create_test_telegram_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Timer trigger node
        nodes.push(WorkflowNode {
            id: "timer_1".to_string(),
            node_type: "timer".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("interval".to_string(), ConfigValue::Number(60000.0)); // 1 minute
                    params
                },
            },
        });

        // Check price node
        nodes.push(WorkflowNode {
            id: "check_price_1".to_string(),
            node_type: "check-price".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), ConfigValue::String("BTC".to_string()));
                    params
                },
            },
        });

        // Telegram notification node
        nodes.push(WorkflowNode {
            id: "telegram_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
                    params.insert("bot_token".to_string(), ConfigValue::String("test_bot_token".to_string()));
                    params.insert("chat_id".to_string(), ConfigValue::String("test_chat_id".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("BTC price update: ${price}".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "timer_1".to_string(),
            target: "check_price_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "check_price_1".to_string(),
            target: "telegram_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        WorkflowData {
            name: "Telegram Price Alert Workflow".to_string(),
            description: Some("Sends BTC price updates to Telegram".to_string()),
            nodes,
            edges,
            trigger_type: "timer".to_string(),
            is_active: true,
        }
    }

    fn create_test_discord_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Manual trigger node
        nodes.push(WorkflowNode {
            id: "trigger_1".to_string(),
            node_type: "manual".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        });

        // Check balance node
        nodes.push(WorkflowNode {
            id: "check_balance_1".to_string(),
            node_type: "check-balance".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("chain".to_string(), ConfigValue::String("ethereum".to_string()));
                    params.insert("token".to_string(), ConfigValue::String("ETH".to_string()));
                    params
                },
            },
        });

        // Discord notification node
        nodes.push(WorkflowNode {
            id: "discord_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
                    params.insert("webhook_url".to_string(), ConfigValue::String("https://discord.com/api/webhooks/test".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("ETH balance: ${balance}".to_string()));
                    params.insert("username".to_string(), ConfigValue::String("DeFlow Bot".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "trigger_1".to_string(),
            target: "check_balance_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "check_balance_1".to_string(),
            target: "discord_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        WorkflowData {
            name: "Discord Balance Alert Workflow".to_string(),
            description: Some("Sends ETH balance updates to Discord".to_string()),
            nodes,
            edges,
            trigger_type: "manual".to_string(),
            is_active: true,
        }
    }

    fn create_test_multi_platform_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Timer trigger
        nodes.push(WorkflowNode {
            id: "timer_1".to_string(),
            node_type: "timer".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("interval".to_string(), ConfigValue::Number(300000.0)); // 5 minutes
                    params
                },
            },
        });

        // DeFi yield check
        nodes.push(WorkflowNode {
            id: "yield_check_1".to_string(),
            node_type: "yield-farming-strategy".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("tradingStyle".to_string(), ConfigValue::String("Balanced".to_string()));
                    params.insert("token".to_string(), ConfigValue::String("USDC".to_string()));
                    params.insert("amount".to_string(), ConfigValue::Number(1000.0));
                    params
                },
            },
        });

        // Telegram notification
        nodes.push(WorkflowNode {
            id: "telegram_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 50),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("DeFi yield update: ${apy}% APY found".to_string()));
                    params
                },
            },
        });

        // Discord notification
        nodes.push(WorkflowNode {
            id: "discord_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 150),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("🚀 DeFi Opportunity: ${apy}% APY on ${protocol}".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "timer_1".to_string(),
            target: "yield_check_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "yield_check_1".to_string(),
            target: "telegram_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_3".to_string(),
            source: "yield_check_1".to_string(),
            target: "discord_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        WorkflowData {
            name: "Multi-Platform DeFi Alert Workflow".to_string(),
            description: Some("Sends DeFi yield opportunities to both Telegram and Discord".to_string()),
            nodes,
            edges,
            trigger_type: "timer".to_string(),
            is_active: true,
        }
    }

    fn create_test_price_alert_social_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Timer trigger
        nodes.push(WorkflowNode {
            id: "timer_1".to_string(),
            node_type: "timer".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("interval".to_string(), ConfigValue::Number(60000.0)); // 1 minute
                    params
                },
            },
        });

        // Price check
        nodes.push(WorkflowNode {
            id: "price_check_1".to_string(),
            node_type: "check-price".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("symbol".to_string(), ConfigValue::String("ETH".to_string()));
                    params
                },
            },
        });

        // Condition check
        nodes.push(WorkflowNode {
            id: "condition_1".to_string(),
            node_type: "condition".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("condition".to_string(), ConfigValue::String("price > 3000".to_string()));
                    params
                },
            },
        });

        // Telegram alert
        nodes.push(WorkflowNode {
            id: "telegram_alert_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (700, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("🔥 ETH price alert: ${price} > $3,000!".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "timer_1".to_string(),
            target: "price_check_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "price_check_1".to_string(),
            target: "condition_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_3".to_string(),
            source: "condition_1".to_string(),
            target: "telegram_alert_1".to_string(),
            source_handle: Some("true".to_string()),
            target_handle: None,
        });

        WorkflowData {
            name: "ETH Price Alert Social Workflow".to_string(),
            description: Some("Sends Telegram alert when ETH price exceeds $3,000".to_string()),
            nodes,
            edges,
            trigger_type: "timer".to_string(),
            is_active: true,
        }
    }

    fn create_test_defi_yield_notification_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Event trigger
        nodes.push(WorkflowNode {
            id: "event_trigger_1".to_string(),
            node_type: "event".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("event_type".to_string(), ConfigValue::String("defi_opportunity".to_string()));
                    params
                },
            },
        });

        // Yield farming strategy
        nodes.push(WorkflowNode {
            id: "yield_strategy_1".to_string(),
            node_type: "yield-farming-strategy".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("tradingStyle".to_string(), ConfigValue::String("YieldChaser".to_string()));
                    params.insert("protocol".to_string(), ConfigValue::String("AUTO_SELECT".to_string()));
                    params.insert("chain".to_string(), ConfigValue::String("AUTO_SELECT".to_string()));
                    params.insert("token".to_string(), ConfigValue::String("USDC".to_string()));
                    params.insert("amount".to_string(), ConfigValue::Number(5000.0));
                    params
                },
            },
        });

        // Discord notification with rich formatting
        nodes.push(WorkflowNode {
            id: "discord_rich_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("💰 **DeFi Yield Opportunity Found!**\n\nProtocol: ${protocol}\nChain: ${chain}\nAPY: ${apy}%\nGas Cost: $${gas_cost}".to_string()));
                    params.insert("username".to_string(), ConfigValue::String("DeFlow Yield Bot".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "event_trigger_1".to_string(),
            target: "yield_strategy_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "yield_strategy_1".to_string(),
            target: "discord_rich_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        WorkflowData {
            name: "DeFi Yield Discord Notification Workflow".to_string(),
            description: Some("Sends rich Discord notifications for DeFi yield opportunities".to_string()),
            nodes,
            edges,
            trigger_type: "event".to_string(),
            is_active: true,
        }
    }

    fn create_test_conditional_social_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Manual trigger
        nodes.push(WorkflowNode {
            id: "manual_trigger_1".to_string(),
            node_type: "manual".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        });

        // Check cycles
        nodes.push(WorkflowNode {
            id: "check_cycles_1".to_string(),
            node_type: "check-cycles".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: HashMap::new(),
            },
        });

        // Condition: low cycles
        nodes.push(WorkflowNode {
            id: "low_cycles_condition_1".to_string(),
            node_type: "condition".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("condition".to_string(), ConfigValue::String("cycles < 1000000000".to_string()));
                    params
                },
            },
        });

        // Telegram warning (if low cycles)
        nodes.push(WorkflowNode {
            id: "telegram_warning_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (700, 50),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("⚠️ Low cycles warning: ${cycles} remaining".to_string()));
                    params
                },
            },
        });

        // Discord status update (if cycles OK)
        nodes.push(WorkflowNode {
            id: "discord_status_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (700, 150),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("✅ System healthy: ${cycles} cycles remaining".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "manual_trigger_1".to_string(),
            target: "check_cycles_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "check_cycles_1".to_string(),
            target: "low_cycles_condition_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_3".to_string(),
            source: "low_cycles_condition_1".to_string(),
            target: "telegram_warning_1".to_string(),
            source_handle: Some("true".to_string()),
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_4".to_string(),
            source: "low_cycles_condition_1".to_string(),
            target: "discord_status_1".to_string(),
            source_handle: Some("false".to_string()),
            target_handle: None,
        });

        WorkflowData {
            name: "Conditional Cycles Alert Workflow".to_string(),
            description: Some("Sends different alerts based on cycles status".to_string()),
            nodes,
            edges,
            trigger_type: "manual".to_string(),
            is_active: true,
        }
    }

    fn create_test_error_recovery_workflow() -> WorkflowData {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Timer trigger
        nodes.push(WorkflowNode {
            id: "timer_1".to_string(),
            node_type: "timer".to_string(),
            position: (100, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("interval".to_string(), ConfigValue::Number(120000.0)); // 2 minutes
                    params
                },
            },
        });

        // HTTP request that might fail
        nodes.push(WorkflowNode {
            id: "http_request_1".to_string(),
            node_type: "http_request".to_string(),
            position: (300, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("url".to_string(), ConfigValue::String("https://api.coingecko.com/api/v3/ping".to_string()));
                    params.insert("method".to_string(), ConfigValue::String("GET".to_string()));
                    params
                },
            },
        });

        // Fallback Telegram notification
        nodes.push(WorkflowNode {
            id: "telegram_fallback_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 100),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("telegram".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("⚡ System check completed - API ${status}".to_string()));
                    params
                },
            },
        });

        // Error notification via Discord
        nodes.push(WorkflowNode {
            id: "discord_error_1".to_string(),
            node_type: "social-media-post".to_string(),
            position: (500, 200),
            configuration: NodeConfiguration {
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("platform".to_string(), ConfigValue::String("discord".to_string()));
                    params.insert("message".to_string(), ConfigValue::String("🚨 API Error Detected: ${error_message}".to_string()));
                    params
                },
            },
        });

        // Connect nodes
        edges.push(crate::types::WorkflowEdge {
            id: "edge_1".to_string(),
            source: "timer_1".to_string(),
            target: "http_request_1".to_string(),
            source_handle: None,
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_2".to_string(),
            source: "http_request_1".to_string(),
            target: "telegram_fallback_1".to_string(),
            source_handle: Some("success".to_string()),
            target_handle: None,
        });

        edges.push(crate::types::WorkflowEdge {
            id: "edge_3".to_string(),
            source: "http_request_1".to_string(),
            target: "discord_error_1".to_string(),
            source_handle: Some("error".to_string()),
            target_handle: None,
        });

        WorkflowData {
            name: "Error Recovery Social Workflow".to_string(),
            description: Some("Demonstrates error handling with social media notifications".to_string()),
            nodes,
            edges,
            trigger_type: "timer".to_string(),
            is_active: true,
        }
    }
}