// Social Media Text Formatting Module
// Enhanced JSON text formatting for social posts with DeFi execution details

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

use super::price_alert_service::{PriceAlert, TokenPrice, SocialPlatform};
use super::price_alert_defi_integration::{DeFiExecutionResult, DailyExecutionStats};

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SocialMediaTemplate {
    pub platform: SocialPlatform,
    pub message_type: SocialMessageType,
    pub template: String,
    pub max_length: Option<u32>,
    pub include_media: bool,
    pub hashtags: Vec<String>,
    pub variables: Vec<String>, // Available template variables
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SocialMessageType {
    PriceAlert,
    DeFiExecution,
    PortfolioUpdate,
    MarketAnalysis,
    TradingSignal,
    CommunityUpdate,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SocialPostData {
    pub platform: SocialPlatform,
    pub message: String,
    pub media_urls: Vec<String>,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
    pub json_payload: String, // Structured JSON for API consumption
    pub scheduled_time: Option<u64>,
    pub priority: SocialPostPriority,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum SocialPostPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiSocialContext {
    pub strategy_executed: Option<String>,
    pub execution_result: Option<DeFiExecutionResult>,
    pub profit_loss: Option<f64>,
    pub portfolio_impact: Option<f64>,
    pub risk_level: Option<u8>,
    pub execution_time: Option<u64>,
}

pub struct SocialMediaFormatter {
    templates: HashMap<(SocialPlatform, SocialMessageType), SocialMediaTemplate>,
    platform_configs: HashMap<SocialPlatform, PlatformConfig>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub max_message_length: u32,
    pub hashtag_limit: u32,
    pub media_support: bool,
    pub thread_support: bool,
    pub emoji_support: bool,
}

impl SocialMediaFormatter {
    pub fn new() -> Self {
        let mut formatter = Self {
            templates: HashMap::new(),
            platform_configs: HashMap::new(),
        };
        
        formatter.initialize_default_templates();
        formatter.initialize_platform_configs();
        formatter
    }

    fn initialize_platform_configs(&mut self) {
        // Twitter/X configuration
        self.platform_configs.insert(SocialPlatform::Twitter, PlatformConfig {
            max_message_length: 280,
            hashtag_limit: 3,
            media_support: true,
            thread_support: true,
            emoji_support: true,
        });

        // Discord configuration
        self.platform_configs.insert(SocialPlatform::Discord, PlatformConfig {
            max_message_length: 2000,
            hashtag_limit: 10,
            media_support: true,
            thread_support: false,
            emoji_support: true,
        });

        // Telegram configuration
        self.platform_configs.insert(SocialPlatform::Telegram, PlatformConfig {
            max_message_length: 4096,
            hashtag_limit: 5,
            media_support: true,
            thread_support: false,
            emoji_support: true,
        });

        // Reddit configuration
        self.platform_configs.insert(SocialPlatform::Reddit, PlatformConfig {
            max_message_length: 40000,
            hashtag_limit: 15,
            media_support: true,
            thread_support: false,
            emoji_support: true,
        });
    }

    fn initialize_default_templates(&mut self) {
        // Twitter Price Alert Templates
        self.add_template(SocialPlatform::Twitter, SocialMessageType::PriceAlert, 
            "🚨 {token} Alert! 
📈 Price: ${price} ({change_24h:+.2}% 24h)
⚡ Condition: {condition}
🎯 Strategy: {defi_action}
💰 P&L: {profit_loss}

{hashtags} 

🤖 via @DeFlowProtocol".to_string()
        );

        self.add_template(SocialPlatform::Twitter, SocialMessageType::DeFiExecution,
            "⚡ DeFi Execution Alert!

🎯 Strategy: {strategy_type}
💎 Token: {token}
💰 Amount: ${amount}
📊 Result: {execution_status}
🔥 ROI: {roi_percentage:+.2}%

{hashtags}

🤖 Automated via @DeFlowProtocol".to_string()
        );

        // Discord Price Alert Templates
        self.add_template(SocialPlatform::Discord, SocialMessageType::PriceAlert,
            "🚨 **PRICE ALERT TRIGGERED** 🚨

**{token}** has reached your target!
📈 **Current Price:** ${price}
📊 **24h Change:** {change_24h:+.2}%
⚡ **Condition Met:** {condition}
🎯 **DeFi Action:** {defi_action}
💰 **Estimated P&L:** {profit_loss}
⏰ **Time:** {timestamp}

{execution_details}

🤖 *Automated alert from DeFlow Protocol*".to_string()
        );

        // Telegram Templates
        self.add_template(SocialPlatform::Telegram, SocialMessageType::PriceAlert,
            "🚨 <b>DeFlow Price Alert</b> 🚨

💎 <b>{token}</b> → <code>${price}</code>
📊 24h Change: <b>{change_24h:+.2}%</b>
⚡ Trigger: <i>{condition}</i>

🎯 <b>DeFi Strategy Executed:</b>
• Type: {strategy_type}
• Amount: ${amount}
• Status: {execution_status}
• P&L: <b>{profit_loss}</b>

⏰ {timestamp}

{hashtags}".to_string()
        );

        // Reddit Templates (more detailed)
        self.add_template(SocialPlatform::Reddit, SocialMessageType::PriceAlert,
            "# 🚨 DeFlow Price Alert & DeFi Execution Report

## Price Action Summary
- **Token:** {token}
- **Current Price:** ${price}
- **24h Change:** {change_24h:+.2}%
- **Condition Met:** {condition}

## Automated DeFi Execution
- **Strategy Type:** {strategy_type}
- **Execution Amount:** ${amount}
- **Execution Status:** {execution_status}
- **Transaction Hash:** {tx_hash}
- **Estimated ROI:** {roi_percentage:+.2}%
- **Gas Cost:** ${gas_cost}

## Portfolio Impact
{portfolio_summary}

## Market Context
{market_analysis}

---
*This alert was generated automatically by [DeFlow Protocol](https://deflow.ai) - Automated DeFi strategies triggered by price conditions.*

{hashtags}".to_string()
        );
    }

    fn add_template(&mut self, platform: SocialPlatform, message_type: SocialMessageType, template: String) {
        let variables = self.extract_template_variables(&template);
        let config = self.platform_configs.get(&platform).cloned().unwrap_or_default();
        
        let social_template = SocialMediaTemplate {
            platform: platform.clone(),
            message_type: message_type.clone(),
            template,
            max_length: Some(config.max_message_length),
            include_media: config.media_support,
            hashtags: self.get_default_hashtags(&message_type),
            variables,
        };

        self.templates.insert((platform, message_type), social_template);
    }

    fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    var_name.push(chars.next().unwrap());
                }
                if !var_name.is_empty() && !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }
        
        variables
    }

    fn get_default_hashtags(&self, message_type: &SocialMessageType) -> Vec<String> {
        match message_type {
            SocialMessageType::PriceAlert => vec![
                "DeFi".to_string(),
                "PriceAlert".to_string(),
                "Crypto".to_string(),
                "AutomatedTrading".to_string(),
            ],
            SocialMessageType::DeFiExecution => vec![
                "DeFi".to_string(),
                "YieldFarming".to_string(),
                "AutomatedStrategy".to_string(),
                "SmartContract".to_string(),
            ],
            SocialMessageType::PortfolioUpdate => vec![
                "Portfolio".to_string(),
                "DeFiManagement".to_string(),
                "YieldOptimization".to_string(),
            ],
            _ => vec!["DeFlow".to_string(), "DeFi".to_string()],
        }
    }

    /// Format a social media post for price alerts with DeFi execution details
    pub fn format_price_alert_post(
        &self,
        platform: &SocialPlatform,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        defi_context: Option<&DeFiSocialContext>,
        custom_hashtags: Option<&[String]>,
    ) -> Result<SocialPostData, String> {
        let template = self.templates
            .get(&(platform.clone(), SocialMessageType::PriceAlert))
            .ok_or("No template found for platform and message type")?;

        let mut context = self.build_context_map(alert, current_price, defi_context);
        let message = self.apply_template(&template.template, &mut context)?;
        
        let hashtags = if let Some(custom) = custom_hashtags {
            custom.to_vec()
        } else {
            template.hashtags.clone()
        };

        let json_payload = self.create_json_payload(alert, current_price, defi_context, &hashtags)?;
        
        Ok(SocialPostData {
            platform: platform.clone(),
            message,
            media_urls: vec![],
            hashtags,
            mentions: vec!["@DeFlowProtocol".to_string()],
            json_payload,
            scheduled_time: None,
            priority: self.determine_priority(alert, defi_context),
        })
    }

    fn build_context_map(
        &self,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        defi_context: Option<&DeFiSocialContext>,
    ) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        // Basic price alert context
        context.insert("token".to_string(), alert.token_symbol.clone());
        context.insert("price".to_string(), format!("{:.4}", current_price.price_usd));
        context.insert("change_24h".to_string(), format!("{:.2}", current_price.change_24h));
        context.insert("condition".to_string(), self.format_condition_text(&alert.condition));
        context.insert("timestamp".to_string(), self.format_timestamp_readable(ic_cdk::api::time()));
        context.insert("volume_24h".to_string(), format!("{:.0}", current_price.volume_24h));
        context.insert("market_cap".to_string(), format!("{:.0}", current_price.market_cap));
        
        // DeFi execution context
        if let Some(defi) = defi_context {
            context.insert("strategy_type".to_string(), 
                defi.strategy_executed.clone().unwrap_or("N/A".to_string()));
            
            if let Some(result) = &defi.execution_result {
                context.insert("execution_status".to_string(), 
                    if result.success { "✅ Success".to_string() } else { "❌ Failed".to_string() });
                context.insert("tx_hash".to_string(), 
                    result.transaction_hash.clone().unwrap_or("Pending".to_string()));
                context.insert("gas_cost".to_string(), 
                    format!("{:.2}", result.actual_gas_cost.unwrap_or(0.0)));
            }
            
            context.insert("profit_loss".to_string(), 
                if let Some(pl) = defi.profit_loss {
                    if pl >= 0.0 { format!("+${:.2}", pl) } else { format!("-${:.2}", pl.abs()) }
                } else { "Pending".to_string() });
                
            context.insert("roi_percentage".to_string(), 
                format!("{:.2}", defi.profit_loss.unwrap_or(0.0)));
            
            context.insert("risk_level".to_string(), 
                format!("{}/10", defi.risk_level.unwrap_or(5)));
        } else {
            context.insert("defi_action".to_string(), "None".to_string());
            context.insert("profit_loss".to_string(), "N/A".to_string());
            context.insert("execution_status".to_string(), "N/A".to_string());
        }
        
        context
    }

    fn apply_template(&self, template: &str, context: &HashMap<String, String>) -> Result<String, String> {
        let mut result = template.to_string();
        
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        // Clean up any remaining unreplaced placeholders
        result = result.replace("{hashtags}", "");
        result = result.replace("{execution_details}", "");
        result = result.replace("{portfolio_summary}", "");
        result = result.replace("{market_analysis}", "");
        
        Ok(result)
    }

    fn create_json_payload(
        &self,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        defi_context: Option<&DeFiSocialContext>,
        hashtags: &[String],
    ) -> Result<String, String> {
        // Create structured JSON payload for API consumption
        let payload = format!(
            r#"{{
  "type": "price_alert",
  "alert_id": "{}",
  "user_id": "{}",
  "token": {{
    "symbol": "{}",
    "price_usd": {},
    "change_24h": {},
    "volume_24h": {},
    "market_cap": {},
    "source": "{:?}"
  }},
  "condition": "{}",
  "defi_execution": {},
  "hashtags": [{}],
  "timestamp": {},
  "priority": "normal"
}}"#,
            alert.id,
            alert.user_id,
            alert.token_symbol,
            current_price.price_usd,
            current_price.change_24h,
            current_price.volume_24h,
            current_price.market_cap,
            current_price.source,
            self.format_condition_text(&alert.condition),
            self.format_defi_execution_json(defi_context),
            hashtags.iter().map(|h| format!("\"{}\"", h)).collect::<Vec<_>>().join(", "),
            ic_cdk::api::time()
        );
        
        Ok(payload)
    }

    fn format_defi_execution_json(&self, defi_context: Option<&DeFiSocialContext>) -> String {
        if let Some(defi) = defi_context {
            if let Some(result) = &defi.execution_result {
                format!(
                    r#"{{
    "strategy_type": "{}",
    "success": {},
    "transaction_hash": "{}",
    "estimated_return": {},
    "gas_cost": {},
    "profit_loss": {},
    "risk_level": {}
  }}"#,
                    defi.strategy_executed.as_ref().unwrap_or(&"unknown".to_string()),
                    result.success,
                    result.transaction_hash.as_ref().unwrap_or(&"pending".to_string()),
                    result.estimated_return.unwrap_or(0.0),
                    result.actual_gas_cost.unwrap_or(0.0),
                    defi.profit_loss.unwrap_or(0.0),
                    defi.risk_level.unwrap_or(5)
                )
            } else {
                "null".to_string()
            }
        } else {
            "null".to_string()
        }
    }

    fn format_condition_text(&self, condition: &super::price_alert_service::PriceCondition) -> String {
        use super::price_alert_service::PriceCondition;
        match condition {
            PriceCondition::Above(price) => format!("above ${:.4}", price),
            PriceCondition::Below(price) => format!("below ${:.4}", price),
            PriceCondition::PercentChange { change_percent, timeframe_minutes, .. } => {
                format!("{}% change in {} min", change_percent, timeframe_minutes)
            }
        }
    }

    fn format_timestamp_readable(&self, timestamp: u64) -> String {
        let seconds = timestamp / 1_000_000_000;
        // Simple timestamp formatting - in production, use proper datetime formatting
        format!("T+{}", seconds % 86400) // Show seconds in day
    }

    fn determine_priority(&self, alert: &PriceAlert, defi_context: Option<&DeFiSocialContext>) -> SocialPostPriority {
        if let Some(defi) = defi_context {
            if let Some(pl) = defi.profit_loss {
                if pl.abs() > 10000.0 { // Major P&L movements
                    return SocialPostPriority::Critical;
                } else if pl.abs() > 1000.0 {
                    return SocialPostPriority::High;
                }
            }
        }
        
        // Check if it's a high-value alert
        if alert.actions.len() > 2 {
            SocialPostPriority::High
        } else {
            SocialPostPriority::Normal
        }
    }

    /// Get available templates for a platform
    pub fn get_templates_for_platform(&self, platform: &SocialPlatform) -> Vec<&SocialMediaTemplate> {
        self.templates
            .iter()
            .filter(|((p, _), _)| p == platform)
            .map(|(_, template)| template)
            .collect()
    }

    /// Create a custom template
    pub fn create_custom_template(
        &mut self,
        platform: SocialPlatform,
        message_type: SocialMessageType,
        template: String,
    ) -> Result<(), String> {
        self.add_template(platform, message_type, template);
        Ok(())
    }
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            max_message_length: 1000,
            hashtag_limit: 5,
            media_support: false,
            thread_support: false,
            emoji_support: true,
        }
    }
}

// Global formatter instance
use std::cell::RefCell;
thread_local! {
    static SOCIAL_FORMATTER: RefCell<SocialMediaFormatter> = RefCell::new(SocialMediaFormatter::new());
}

/// Format a social media post with DeFi context
pub fn format_social_post_with_defi(
    platform: &SocialPlatform,
    alert: &PriceAlert,
    current_price: &TokenPrice,
    defi_result: Option<&DeFiExecutionResult>,
    custom_hashtags: Option<&[String]>,
) -> Result<SocialPostData, String> {
    let defi_context = defi_result.map(|result| DeFiSocialContext {
        strategy_executed: Some("market_order".to_string()), // Would be determined from actual execution
        execution_result: Some(result.clone()),
        profit_loss: result.estimated_return,
        portfolio_impact: None,
        risk_level: Some(5),
        execution_time: Some(ic_cdk::api::time()),
    });

    SOCIAL_FORMATTER.with(|formatter| {
        formatter.borrow().format_price_alert_post(
            platform,
            alert,
            current_price,
            defi_context.as_ref(),
            custom_hashtags,
        )
    })
}

/// Get all available social media templates
pub fn get_available_templates() -> Vec<SocialMediaTemplate> {
    SOCIAL_FORMATTER.with(|formatter| {
        formatter.borrow().templates.values().cloned().collect()
    })
}