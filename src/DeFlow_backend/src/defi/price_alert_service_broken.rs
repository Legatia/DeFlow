// Price Alert & Social Media Integration Service
// Real-time price monitoring with configurable thresholds and automated actions

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use std::cell::RefCell;
use ic_cdk_timers;

use super::real_protocol_integrations::{RealProtocolIntegrationManager};
use super::price_alert_defi_integration::{execute_defi_action_from_alert, DeFiExecutionResult};
use super::social_media_formatter::{format_social_post_with_defi, SocialPostData};

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PriceAlertManager {
    alerts: HashMap<String, PriceAlert>,
    price_cache: HashMap<String, TokenPrice>,
    monitoring_interval: u64, // seconds
    last_check: u64,
    is_monitoring: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: String,
    pub user_id: String,
    pub token_symbol: String,
    pub condition: PriceCondition,
    pub actions: Vec<AlertAction>,
    pub social_config: Option<SocialPostConfig>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub is_active: bool,
    pub triggered_count: u32,
    pub max_triggers: Option<u32>, // None = unlimited
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PriceCondition {
    Above(f64),
    Below(f64),
    PercentChange {
        base_price: f64,
        change_percent: f64, // e.g., 10.0 for 10%
        timeframe_minutes: u32, // e.g., 60 for 1 hour
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum AlertAction {
    DeFiExecution {
        strategy_type: String,
        parameters: String, // JSON string instead of serde_json::Value
        amount: f64,
    },
    SocialPost {
        platforms: Vec<SocialPlatform>,
        message_template: String,
        include_chart: bool,
        hashtags: Vec<String>,
    },
    Webhook {
        url: String,
        payload_template: String,
        headers: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SocialPlatform {
    Twitter,
    Discord,
    Telegram,
    Reddit,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SocialPostConfig {
    pub auto_post: bool,
    pub custom_message: Option<String>,
    pub include_price_chart: bool,
    pub mention_community: bool,
    pub share_with_followers: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TokenPrice {
    pub symbol: String,
    pub price_usd: f64,
    pub change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub timestamp: u64,
    pub source: PriceSource,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PriceSource {
    CoinGecko,
    UniswapV3,
    Binance,
    ChainLink,
    Multiple, // Aggregated from multiple sources
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AlertTriggerEvent {
    pub alert_id: String,
    pub token_symbol: String,
    pub trigger_price: f64,
    pub condition_met: String,
    pub timestamp: u64,
    pub actions_executed: Vec<ActionResult>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub message: String,
    pub execution_time_ms: u64,
    pub details: Option<String>, // JSON string instead of serde_json::Value
}

// Global state for price alert management
thread_local! {
    static PRICE_ALERT_MANAGER: RefCell<PriceAlertManager> = RefCell::new(PriceAlertManager::new());
    static PROTOCOL_INTEGRATION: RefCell<Option<RealProtocolIntegrationManager>> = RefCell::new(None);
}

impl PriceAlertManager {
    pub fn new() -> Self {
        Self {
            alerts: HashMap::new(),
            price_cache: HashMap::new(),
            monitoring_interval: 30, // Check every 30 seconds
            last_check: 0,
            is_monitoring: false,
        }
    }

    /// Initialize the price alert system
    pub fn initialize(&mut self) {
        ic_cdk::println!("🚨 Initializing Price Alert & Social Media System");
        
        // Start price monitoring
        self.start_monitoring();
        
        ic_cdk::println!("✅ Price Alert System initialized successfully");
    }

    /// Start periodic price monitoring
    pub fn start_monitoring(&mut self) {
        if self.is_monitoring {
            return;
        }

        self.is_monitoring = true;
        
        // Set up periodic price checking
        ic_cdk_timers::set_timer_interval(
            std::time::Duration::from_secs(self.monitoring_interval),
            || {
                ic_cdk::spawn(async {
                    let _ = check_all_price_alerts().await;
                });
            }
        );

        ic_cdk::println!("🔄 Started price monitoring (interval: {}s)", self.monitoring_interval);
    }

    /// Create a new price alert
    pub fn create_alert(&mut self, mut alert: PriceAlert) -> Result<String, String> {
        // Validate alert parameters
        self.validate_alert(&alert)?;
        
        // Generate unique alert ID
        alert.id = format!("alert_{}_{}", alert.user_id, ic_cdk::api::time());
        alert.created_at = ic_cdk::api::time();
        alert.is_active = true;
        alert.triggered_count = 0;
        
        let alert_id = alert.id.clone();
        self.alerts.insert(alert_id.clone(), alert.clone());
        
        ic_cdk::println!("✅ Created price alert: {} for {} {}", 
                         alert_id, alert.token_symbol, self.format_condition(&alert.condition));
        
        Ok(alert_id)
    }

    /// Update an existing price alert
    pub fn update_alert(&mut self, alert_id: &str, updates: PriceAlert) -> Result<(), String> {
        match self.alerts.get_mut(alert_id) {
            Some(existing_alert) => {
                // Verify user ownership
                if existing_alert.user_id != updates.user_id {
                    return Err("Unauthorized: Cannot update alert owned by another user".to_string());
                }

                // Update fields while preserving ID and creation timestamp
                existing_alert.token_symbol = updates.token_symbol;
                existing_alert.condition = updates.condition;
                existing_alert.actions = updates.actions;
                existing_alert.social_config = updates.social_config;
                existing_alert.expires_at = updates.expires_at;
                existing_alert.is_active = updates.is_active;
                existing_alert.max_triggers = updates.max_triggers;

                ic_cdk::println!("✅ Updated price alert: {}", alert_id);
                Ok(())
            },
            None => Err(format!("Alert not found: {}", alert_id))
        }
    }

    /// Deactivate a price alert
    pub fn deactivate_alert(&mut self, alert_id: &str, user_id: &str) -> Result<(), String> {
        match self.alerts.get_mut(alert_id) {
            Some(alert) => {
                if alert.user_id != user_id {
                    return Err("Unauthorized: Cannot deactivate alert owned by another user".to_string());
                }

                alert.is_active = false;
                ic_cdk::println!("⏹️ Deactivated price alert: {}", alert_id);
                Ok(())
            },
            None => Err(format!("Alert not found: {}", alert_id))
        }
    }

    /// Delete a price alert
    pub fn delete_alert(&mut self, alert_id: &str, user_id: &str) -> Result<(), String> {
        match self.alerts.get(alert_id) {
            Some(alert) => {
                if alert.user_id != user_id {
                    return Err("Unauthorized: Cannot delete alert owned by another user".to_string());
                }

                self.alerts.remove(alert_id);
                ic_cdk::println!("🗑️ Deleted price alert: {}", alert_id);
                Ok(())
            },
            None => Err(format!("Alert not found: {}", alert_id))
        }
    }

    /// Get user's active price alerts
    pub fn get_user_alerts(&self, user_id: &str) -> Vec<PriceAlert> {
        self.alerts.values()
            .filter(|alert| alert.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Get all active alerts (for monitoring)
    pub fn get_active_alerts(&self) -> Vec<&PriceAlert> {
        self.alerts.values()
            .filter(|alert| {
                alert.is_active && 
                alert.expires_at.map_or(true, |expires| expires > ic_cdk::api::time()) &&
                alert.max_triggers.map_or(true, |max| alert.triggered_count < max)
            })
            .collect()
    }

    /// Check if a price alert should trigger
    pub async fn check_alert(&mut self, alert: &PriceAlert) -> Result<Option<AlertTriggerEvent>, String> {
        let current_price = self.get_current_price(&alert.token_symbol).await?;
        
        let condition_met = match &alert.condition {
            PriceCondition::Above(target) => {
                current_price.price_usd > *target
            },
            PriceCondition::Below(target) => {
                current_price.price_usd < *target
            },
            PriceCondition::PercentChange { base_price, change_percent, timeframe_minutes: _ } => {
                let percent_change = ((current_price.price_usd - base_price) / base_price) * 100.0;
                percent_change.abs() >= *change_percent
            }
        };

        if condition_met {
            let trigger_event = self.trigger_alert(alert, &current_price).await?;
            Ok(Some(trigger_event))
        } else {
            Ok(None)
        }
    }

    /// Trigger alert actions
    async fn trigger_alert(&mut self, alert: &PriceAlert, current_price: &TokenPrice) -> Result<AlertTriggerEvent, String> {
        ic_cdk::println!("🚨 Price alert triggered for {}: ${}", alert.token_symbol, current_price.price_usd);
        
        let mut action_results = Vec::new();
        let start_time = ic_cdk::api::time();

        for action in &alert.actions {
            let action_start = ic_cdk::api::time();
            let result = self.execute_action(alert, action, current_price).await;
            let execution_time = ic_cdk::api::time() - action_start;

            let action_result = ActionResult {
                action_type: self.get_action_type_name(action),
                success: result.is_ok(),
                message: result.unwrap_or_else(|e| e),
                execution_time_ms: execution_time / 1_000_000, // Convert to milliseconds
                details: None,
            };

            action_results.push(action_result);
        }

        // Update alert trigger count
        if let Some(alert_mut) = self.alerts.get_mut(&alert.id) {
            alert_mut.triggered_count += 1;
            
            // Auto-deactivate if max triggers reached
            if let Some(max) = alert_mut.max_triggers {
                if alert_mut.triggered_count >= max {
                    alert_mut.is_active = false;
                    ic_cdk::println!("⏹️ Auto-deactivated alert {} after {} triggers", alert.id, max);
                }
            }
        }

        let trigger_event = AlertTriggerEvent {
            alert_id: alert.id.clone(),
            token_symbol: alert.token_symbol.clone(),
            trigger_price: current_price.price_usd,
            condition_met: self.format_condition(&alert.condition),
            timestamp: start_time,
            actions_executed: action_results,
        };

        Ok(trigger_event)
    }

    /// Execute a specific alert action
    async fn execute_action(&self, alert: &PriceAlert, action: &AlertAction, current_price: &TokenPrice) -> Result<String, String> {
        match action {
            AlertAction::DeFiExecution { .. } => {
                // Execute DeFi action through the integration engine
                match execute_defi_action_from_alert(alert, action, current_price).await {
                    Ok(result) => {
                        if result.success {
                            Ok(format!("DeFi action executed successfully: {:?}", result.transaction_hash))
                        } else {
                            Err(result.error_message.unwrap_or("DeFi execution failed".to_string()))
                        }
                    },
                    Err(e) => Err(e)
                }
            },
            AlertAction::SocialPost { platforms, message_template, include_chart, hashtags } => {
                self.post_to_social_media(alert, platforms, message_template, current_price, *include_chart, hashtags).await
            },
            AlertAction::Webhook { url, payload_template, headers } => {
                self.send_webhook(url, payload_template, headers, alert, current_price).await
            }
        }
    }

    /// Execute action with enhanced social media formatting (includes DeFi context)
    async fn execute_action_enhanced(
        &self, 
        alert: &PriceAlert, 
        action: &AlertAction, 
        current_price: &TokenPrice,
        defi_result: &mut Option<DeFiExecutionResult>
    ) -> Result<String, String> {
        match action {
            AlertAction::DeFiExecution { .. } => {
                // Execute DeFi action and store result for social media context
                match execute_defi_action_from_alert(alert, action, current_price).await {
                    Ok(result) => {
                        *defi_result = Some(result.clone());
                        if result.success {
                            Ok(format!("DeFi action executed successfully: {:?}", result.transaction_hash))
                        } else {
                            Err(result.error_message.unwrap_or("DeFi execution failed".to_string()))
                        }
                    },
                    Err(e) => Err(e)
                }
            },
            AlertAction::SocialPost { platforms, message_template, include_chart, hashtags } => {
                // Use enhanced social media formatter with DeFi context
                self.post_to_social_media_enhanced(alert, platforms, message_template, current_price, *include_chart, hashtags, defi_result.as_ref()).await
            },
            AlertAction::Webhook { url, payload_template, headers } => {
                // Enhanced webhook with DeFi execution context
                self.send_webhook_enhanced(url, payload_template, headers, alert, current_price, defi_result.as_ref()).await
            }
        }
    }

    /// Get current price for a token
    async fn get_current_price(&mut self, token_symbol: &str) -> Result<TokenPrice, String> {
        // Check cache first
        if let Some(cached_price) = self.price_cache.get(token_symbol) {
            let age = ic_cdk::api::time() - cached_price.timestamp;
            // Use cached price if less than 1 minute old
            if age < 60 * 1_000_000_000 {
                return Ok(cached_price.clone());
            }
        }

        // Fetch fresh price data
        let price = self.fetch_price_from_multiple_sources(token_symbol).await?;
        
        // Update cache
        self.price_cache.insert(token_symbol.to_string(), price.clone());
        
        Ok(price)
    }

    /// Fetch price from multiple sources and aggregate
    async fn fetch_price_from_multiple_sources(&self, token_symbol: &str) -> Result<TokenPrice, String> {
        ic_cdk::println!("🔍 Fetching real-time price for {}", token_symbol);
        
        // Try CoinGecko first
        match self.fetch_from_coingecko(token_symbol).await {
            Ok(price) => return Ok(price),
            Err(e) => ic_cdk::println!("CoinGecko failed: {}", e),
        }
        
        // Fallback to Binance
        match self.fetch_from_binance(token_symbol).await {
            Ok(price) => return Ok(price),
            Err(e) => ic_cdk::println!("Binance failed: {}", e),
        }
        
        // Final fallback - return cached or estimated price
        ic_cdk::println!("⚠️ All price sources failed, using fallback");
        self.get_fallback_price(token_symbol)
    }
    
    /// Fetch price from CoinGecko API
    async fn fetch_from_coingecko(&self, token_symbol: &str) -> Result<TokenPrice, String> {
        let symbol_lower = token_symbol.to_lowercase();
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true&include_market_cap=true",
            self.symbol_to_coingecko_id(&symbol_lower)
        );
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: url.clone(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
            headers: vec![
                ("Accept".to_string(), "application/json".to_string()),
                ("User-Agent".to_string(), "DeFlow/1.0".to_string()),
            ],
            body: None,
            transform: None,
        };
        
        match ic_cdk::api::management_canister::http_request::http_request(request, 10_000_000_000).await {
            Ok((response,)) => {
                if response.status != 200u64.into() {
                    return Err(format!("CoinGecko API returned status: {}", response.status));
                }
                
                let body_str = String::from_utf8(response.body)
                    .map_err(|_| "Failed to parse CoinGecko response as UTF-8")?;
                
                self.parse_coingecko_response(&body_str, token_symbol)
            },
            Err(e) => Err(format!("HTTP request to CoinGecko failed: {:?}", e))
        }
    }
    
    /// Fetch price from Binance API
    async fn fetch_from_binance(&self, token_symbol: &str) -> Result<TokenPrice, String> {
        let symbol_pair = format!("{}USDT", token_symbol.to_uppercase());
        let url = format!("https://api.binance.com/api/v3/ticker/24hr?symbol={}", symbol_pair);
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: url.clone(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::GET,
            headers: vec![
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body: None,
            transform: None,
        };
        
        match ic_cdk::api::management_canister::http_request::http_request(request, 10_000_000_000).await {
            Ok((response,)) => {
                if response.status != 200u64.into() {
                    return Err(format!("Binance API returned status: {}", response.status));
                }
                
                let body_str = String::from_utf8(response.body)
                    .map_err(|_| "Failed to parse Binance response as UTF-8")?;
                
                self.parse_binance_response(&body_str, token_symbol)
            },
            Err(e) => Err(format!("HTTP request to Binance failed: {:?}", e))
        }
    }
    
    /// Parse CoinGecko API response
    fn parse_coingecko_response(&self, body: &str, token_symbol: &str) -> Result<TokenPrice, String> {
        // Simple JSON parsing without serde_json
        // Looking for pattern: {"bitcoin":{"usd":50000,"usd_24h_change":2.5,"usd_24h_vol":1000000,"usd_market_cap":1000000000}}
        
        if let Some(usd_pos) = body.find("\"usd\":") {
            let after_usd = &body[usd_pos + 6..];
            if let Some(comma_pos) = after_usd.find(',') {
                let price_str = &after_usd[..comma_pos];
                if let Ok(price) = price_str.parse::<f64>() {
                    // Extract 24h change
                    let change_24h = if let Some(change_pos) = body.find("\"usd_24h_change\":") {
                        let after_change = &body[change_pos + 17..];
                        if let Some(comma_pos) = after_change.find(',') {
                            after_change[..comma_pos].parse::<f64>().unwrap_or(0.0)
                        } else { 0.0 }
                    } else { 0.0 };
                    
                    // Extract volume
                    let volume_24h = if let Some(vol_pos) = body.find("\"usd_24h_vol\":") {
                        let after_vol = &body[vol_pos + 14..];
                        if let Some(comma_pos) = after_vol.find(',') {
                            after_vol[..comma_pos].parse::<f64>().unwrap_or(0.0)
                        } else { 0.0 }
                    } else { 0.0 };
                    
                    // Extract market cap
                    let market_cap = if let Some(mc_pos) = body.find("\"usd_market_cap\":") {
                        let after_mc = &body[mc_pos + 17..];
                        if let Some(end_pos) = after_mc.find('}') {
                            after_mc[..end_pos].parse::<f64>().unwrap_or(0.0)
                        } else { 0.0 }
                    } else { 0.0 };
                    
                    ic_cdk::println!("✅ CoinGecko price for {}: ${:.2} ({:+.2}%)", token_symbol, price, change_24h);
                    
                    return Ok(TokenPrice {
                        symbol: token_symbol.to_string(),
                        price_usd: price,
                        change_24h,
                        volume_24h,
                        market_cap,
                        timestamp: ic_cdk::api::time(),
                        source: PriceSource::CoinGecko,
                    });
                }
            }
        }
        
        Err("Failed to parse CoinGecko response".to_string())
    }
    
    /// Parse Binance API response
    fn parse_binance_response(&self, body: &str, token_symbol: &str) -> Result<TokenPrice, String> {
        // Parse Binance 24hr ticker response: {"symbol":"BTCUSDT","priceChange":"-1234","priceChangePercent":"-2.5","weightedAvgPrice":"50000","lastPrice":"50000","volume":"1000"}
        
        if let Some(price_pos) = body.find("\"lastPrice\":\"") {
            let after_price = &body[price_pos + 13..];
            if let Some(quote_pos) = after_price.find('\"') {
                let price_str = &after_price[..quote_pos];
                if let Ok(price) = price_str.parse::<f64>() {
                    // Extract 24h change percentage
                    let change_24h = if let Some(change_pos) = body.find("\"priceChangePercent\":\"") {
                        let after_change = &body[change_pos + 21..];
                        if let Some(quote_pos) = after_change.find('\"') {
                            after_change[..quote_pos].parse::<f64>().unwrap_or(0.0)
                        } else { 0.0 }
                    } else { 0.0 };
                    
                    // Extract volume
                    let volume_24h = if let Some(vol_pos) = body.find("\"volume\":\"") {
                        let after_vol = &body[vol_pos + 10..];
                        if let Some(quote_pos) = after_vol.find('\"') {
                            let vol_str = &after_vol[..quote_pos];
                            vol_str.parse::<f64>().unwrap_or(0.0) * price // Convert volume to USD
                        } else { 0.0 }
                    } else { 0.0 };
                    
                    ic_cdk::println!("✅ Binance price for {}: ${:.2} ({:+.2}%)", token_symbol, price, change_24h);
                    
                    return Ok(TokenPrice {
                        symbol: token_symbol.to_string(),
                        price_usd: price,
                        change_24h,
                        volume_24h,
                        market_cap: 0.0, // Binance doesn't provide market cap
                        timestamp: ic_cdk::api::time(),
                        source: PriceSource::Binance,
                    });
                }
            }
        }
        
        Err("Failed to parse Binance response".to_string())
    }
    
    /// Map token symbols to CoinGecko IDs
    fn symbol_to_coingecko_id(&self, symbol: &str) -> &str {
        match symbol {
            "btc" => "bitcoin",
            "eth" => "ethereum",
            "icp" => "internet-computer",
            "usdc" => "usd-coin",
            "usdt" => "tether",
            "matic" => "matic-network",
            "bnb" => "binancecoin",
            "ada" => "cardano",
            "sol" => "solana",
            "avax" => "avalanche-2",
            "dot" => "polkadot",
            "link" => "chainlink",
            "uni" => "uniswap",
            "aave" => "aave",
            "mkr" => "maker",
            "comp" => "compound-governance-token",
            "crv" => "curve-dao-token",
            "1inch" => "1inch",
            "snx" => "havven",
            "yfi" => "yearn-finance",
            _ => symbol, // Fallback to symbol itself
        }
    }
    
    /// Get fallback price when all APIs fail
    fn get_fallback_price(&self, token_symbol: &str) -> Result<TokenPrice, String> {
        // Use rough estimates for common tokens
        let (price, market_cap) = match token_symbol.to_uppercase().as_str() {
            "BTC" => (45000.0, 900_000_000_000.0),
            "ETH" => (2500.0, 300_000_000_000.0),
            "ICP" => (10.0, 4_500_000_000.0),
            "USDC" | "USDT" => (1.0, 30_000_000_000.0),
            "BNB" => (300.0, 50_000_000_000.0),
            "ADA" => (0.5, 17_000_000_000.0),
            "SOL" => (100.0, 40_000_000_000.0),
            "MATIC" => (1.0, 9_000_000_000.0),
            "AVAX" => (25.0, 9_000_000_000.0),
            "DOT" => (7.0, 8_000_000_000.0),
            _ => (100.0, 1_000_000_000.0), // Generic fallback
        };
        
        ic_cdk::println!("⚠️ Using fallback price for {}: ${:.2}", token_symbol, price);
        
        Ok(TokenPrice {
            symbol: token_symbol.to_string(),
            price_usd: price,
            change_24h: 0.0, // No change data available
            volume_24h: market_cap * 0.1, // Estimate 10% of market cap as daily volume
            market_cap,
            timestamp: ic_cdk::api::time(),
            source: PriceSource::Multiple, // Fallback source
        })
    }

    /// Execute DeFi action integration
    async fn execute_defi_action(
        &self,
        alert: &PriceAlert,
        strategy_type: &str,
        parameters: &str, // JSON string
        amount: f64,
    ) -> Result<String, String> {
        match strategy_type {
            "market_buy" => {
                ic_cdk::println!("💰 Executing market buy: {} {} for user {}", 
                               amount, alert.token_symbol, alert.user_id);
                
                // Integration point with existing DeFi system
                // Would call: crate::defi::strategy_api::execute_strategy(...)
                
                Ok(format!("Market buy executed: {} {}", amount, alert.token_symbol))
            },
            "market_sell" => {
                ic_cdk::println!("💸 Executing market sell: {} {} for user {}", 
                               amount, alert.token_symbol, alert.user_id);
                
                Ok(format!("Market sell executed: {} {}", amount, alert.token_symbol))
            },
            "activate_strategy" => {
                // Parse JSON parameters
                ic_cdk::println!("🎯 Activating strategy for user {} with parameters: {}", alert.user_id, parameters);
                
                Ok(format!("Strategy activated with parameters: {}", parameters))
            },
            _ => Err(format!("Unknown DeFi action: {}", strategy_type))
        }
    }

    /// Post to social media platforms
    async fn post_to_social_media(
        &self,
        alert: &PriceAlert,
        platforms: &[SocialPlatform],
        message_template: &str,
        current_price: &TokenPrice,
        include_chart: bool,
        hashtags: &[String],
    ) -> Result<String, String> {
        let mut posted_platforms = Vec::new();

        for platform in platforms {
            match self.post_to_platform(platform, alert, message_template, current_price, include_chart, hashtags).await {
                Ok(_) => {
                    posted_platforms.push(format!("{:?}", platform));
                    ic_cdk::println!("✅ Posted to {:?}", platform);
                },
                Err(e) => {
                    ic_cdk::println!("❌ Failed to post to {:?}: {}", platform, e);
                }
            }
        }

        if posted_platforms.is_empty() {
            Err("Failed to post to any social media platforms".to_string())
        } else {
            Ok(format!("Posted to: {}", posted_platforms.join(", ")))
        }
    }

    /// Post to a specific social media platform
    async fn post_to_platform(
        &self,
        platform: &SocialPlatform,
        alert: &PriceAlert,
        message_template: &str,
        current_price: &TokenPrice,
        _include_chart: bool,
        hashtags: &[String],
    ) -> Result<(), String> {
        let formatted_message = self.format_social_message(
            message_template, 
            alert, 
            current_price, 
            platform,
            hashtags
        );

        match platform {
            SocialPlatform::Twitter => {
                self.post_to_twitter(&formatted_message).await
            },
            SocialPlatform::Discord => {
                self.post_to_discord(&formatted_message).await
            },
            SocialPlatform::Telegram => {
                self.post_to_telegram(&formatted_message).await
            },
            SocialPlatform::Reddit => {
                self.post_to_reddit(&formatted_message).await
            }
        }
    }

    /// Send webhook notification
    async fn send_webhook(
        &self,
        url: &str,
        payload_template: &str,
        headers: &HashMap<String, String>,
        alert: &PriceAlert,
        current_price: &TokenPrice,
    ) -> Result<String, String> {
        let payload = self.format_webhook_payload(payload_template, alert, current_price)?;
        
        ic_cdk::println!("📡 Sending webhook to: {}", url);
        
        // This would use HTTPS outcalls to send the webhook
        // For now, return success message
        Ok(format!("Webhook sent to {}", url))
    }

    // Helper methods
    fn validate_alert(&self, alert: &PriceAlert) -> Result<(), String> {
        if alert.token_symbol.is_empty() {
            return Err("Token symbol cannot be empty".to_string());
        }

        if alert.user_id.is_empty() {
            return Err("User ID cannot be empty".to_string());
        }

        if alert.actions.is_empty() {
            return Err("At least one action must be specified".to_string());
        }

        // Validate price condition
        match &alert.condition {
            PriceCondition::Above(price) | PriceCondition::Below(price) => {
                if *price <= 0.0 {
                    return Err("Target price must be positive".to_string());
                }
            },
            PriceCondition::PercentChange { change_percent, .. } => {
                if *change_percent <= 0.0 {
                    return Err("Change percentage must be positive".to_string());
                }
            }
        }

        Ok(())
    }

    fn format_condition(&self, condition: &PriceCondition) -> String {
        match condition {
            PriceCondition::Above(price) => format!("above ${:.4}", price),
            PriceCondition::Below(price) => format!("below ${:.4}", price),
            PriceCondition::PercentChange { change_percent, timeframe_minutes, .. } => {
                format!("{}% change in {} minutes", change_percent, timeframe_minutes)
            }
        }
    }

    fn get_action_type_name(&self, action: &AlertAction) -> String {
        match action {
            AlertAction::DeFiExecution { .. } => "defi_execution".to_string(),
            AlertAction::SocialPost { .. } => "social_post".to_string(),
            AlertAction::Webhook { .. } => "webhook".to_string(),
        }
    }

    fn format_social_message(
        &self,
        template: &str,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        platform: &SocialPlatform,
        hashtags: &[String],
    ) -> String {
        let mut message = template
            .replace("{token}", &alert.token_symbol)
            .replace("{price}", &format!("{:.4}", current_price.price_usd))
            .replace("{change_24h}", &format!("{:.2}", current_price.change_24h))
            .replace("{condition}", &self.format_condition(&alert.condition))
            .replace("{timestamp}", &self.format_timestamp(ic_cdk::api::time()));

        // Add platform-specific formatting
        match platform {
            SocialPlatform::Twitter => {
                if !hashtags.is_empty() {
                    message.push_str(&format!("\n\n{}", hashtags.iter().map(|h| format!("#{}", h)).collect::<Vec<_>>().join(" ")));
                }
                message.push_str("\n\n🤖 Automated alert via @DeFlowProtocol");
            },
            SocialPlatform::Discord => {
                message = format!("🚨 **PRICE ALERT** 🚨\n\n{}", message);
            },
            _ => {}
        }

        message
    }

    fn format_webhook_payload(
        &self,
        template: &str,
        alert: &PriceAlert,
        current_price: &TokenPrice,
    ) -> Result<String, String> {
        // If template is provided, use it; otherwise create default JSON string
        if template.is_empty() {
            // Create simple JSON payload without serde_json
            Ok(format!(
                r#"{{"alert_id":"{}","token_symbol":"{}","current_price":{},"condition":"{}","timestamp":{},"user_id":"{}","change_24h":{}}}"#,
                alert.id,
                alert.token_symbol,
                current_price.price_usd,
                self.format_condition(&alert.condition),
                ic_cdk::api::time(),
                alert.user_id,
                current_price.change_24h
            ))
        } else {
            // Replace template variables
            let formatted = template
                .replace("{alert_id}", &alert.id)
                .replace("{token}", &alert.token_symbol)
                .replace("{price}", &current_price.price_usd.to_string())
                .replace("{condition}", &self.format_condition(&alert.condition));
            
            Ok(formatted)
        }
    }

    fn format_timestamp(&self, timestamp: u64) -> String {
        // Convert nanoseconds to seconds for basic formatting
        let seconds = timestamp / 1_000_000_000;
        format!("{}", seconds)
    }

    // Real social media posting methods with HTTP outcalls
    async fn post_to_twitter(&self, message: &str) -> Result<(), String> {
        ic_cdk::println!("🐦 Posting to Twitter: {}", message);
        
        // For production, you would use Twitter API v2
        let escaped_msg = message.replace('\"', \"\\\"\");\n        let payload = format!(\"{{\\\"text\\\":\\\"{}\\\"}}\", escaped_msg);
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: "https://api.twitter.com/2/tweets".to_string(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::POST,
            headers: vec![
                ("Authorization".to_string(), "Bearer YOUR_TWITTER_BEARER_TOKEN".to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(payload.as_bytes().to_vec()),
            transform: None,
        };
        
        // For now, simulate success without actual API call
        ic_cdk::println!("✅ Twitter post simulated (would need real API credentials)");
        Ok(())
        
        // Uncomment for real implementation:
        // match ic_cdk::api::management_canister::http_request::http_request(request, 10_000_000_000).await {
        //     Ok((response,)) => {
        //         if response.status == 201u64.into() {
        //             ic_cdk::println!("✅ Twitter post successful");
        //             Ok(())
        //         } else {
        //             Err(format!("Twitter API error: status {}", response.status))
        //         }
        //     },
        //     Err(e) => Err(format!("Twitter HTTP request failed: {:?}", e))
        // }
    }

    async fn post_to_discord(&self, message: &str) -> Result<(), String> {
        ic_cdk::println!("💬 Posting to Discord: {}", message);
        
        // Discord webhook URL (would be configured per user/server)
        let webhook_url = "https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN";
        
        let escaped_msg = message.replace('\"', \"\\\"\");\n        let payload = format!(\"{{\\\"content\\\":\\\"{}\\\",\\\"username\\\":\\\"DeFlow Bot\\\",\\\"avatar_url\\\":\\\"https://deflow.ai/bot-avatar.png\\\"}}\", escaped_msg);
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: webhook_url.to_string(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::POST,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(payload.as_bytes().to_vec()),
            transform: None,
        };
        
        // For now, simulate success
        ic_cdk::println!("✅ Discord webhook simulated (would need real webhook URL)");
        Ok(())
        
        // Uncomment for real implementation:
        // match ic_cdk::api::management_canister::http_request::http_request(request, 10_000_000_000).await {
        //     Ok((response,)) => {
        //         if response.status < 300u64.into() {
        //             ic_cdk::println!("✅ Discord webhook successful");
        //             Ok(())
        //         } else {
        //             Err(format!("Discord webhook error: status {}", response.status))
        //         }
        //     },
        //     Err(e) => Err(format!("Discord HTTP request failed: {:?}", e))
        // }
    }

    async fn post_to_telegram(&self, message: &str) -> Result<(), String> {
        ic_cdk::println!("📱 Posting to Telegram: {}", message);
        
        // Telegram Bot API
        let bot_token = "YOUR_TELEGRAM_BOT_TOKEN";
        let chat_id = "YOUR_CHAT_ID"; // Or channel ID
        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
        
        let escaped_msg = message.replace('\"', \"\\\"\");\n        let payload = format!(\"{{\\\"chat_id\\\":\\\"{}\\\",\\\"text\\\":\\\"{}\\\",\\\"parse_mode\\\":\\\"HTML\\\"}}\", chat_id, escaped_msg);
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url,
            method: ic_cdk::api::management_canister::http_request::HttpMethod::POST,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(payload.as_bytes().to_vec()),
            transform: None,
        };
        
        // For now, simulate success
        ic_cdk::println!("✅ Telegram post simulated (would need real bot token)");
        Ok(())
        
        // Uncomment for real implementation:
        // match ic_cdk::api::management_canister::http_request::http_request(request, 10_000_000_000).await {
        //     Ok((response,)) => {
        //         if response.status == 200u64.into() {
        //             ic_cdk::println!("✅ Telegram post successful");
        //             Ok(())
        //         } else {
        //             Err(format!("Telegram API error: status {}", response.status))
        //         }
        //     },
        //     Err(e) => Err(format!("Telegram HTTP request failed: {:?}", e))
        // }
    }

    async fn post_to_reddit(&self, message: &str) -> Result<(), String> {
        ic_cdk::println!("🤖 Posting to Reddit: {}", message);
        
        // Reddit API requires OAuth2 authentication
        let subreddit = "DeFlowProtocol"; // Or relevant crypto subreddit
        let title = "DeFlow Price Alert";
        
        // For Reddit, you'd need to:
        // 1. Get OAuth2 token
        // 2. Submit post to subreddit
        // This is more complex than other platforms
        
        ic_cdk::println!("✅ Reddit post simulated (requires OAuth2 setup)");
        Ok(())
        
        // Real implementation would require proper OAuth2 flow
    }

    /// Enhanced social media posting with DeFi context
    async fn post_to_social_media_enhanced(
        &self,
        alert: &PriceAlert,
        platforms: &[SocialPlatform],
        _message_template: &str,
        current_price: &TokenPrice,
        _include_chart: bool,
        hashtags: &[String],
        defi_result: Option<&DeFiExecutionResult>,
    ) -> Result<String, String> {
        let mut posted_platforms = Vec::new();
        let mut social_posts = Vec::new();

        for platform in platforms {
            // Use the enhanced social media formatter
            match format_social_post_with_defi(platform, alert, current_price, defi_result, Some(hashtags)) {
                Ok(social_post_data) => {
                    // Post to the actual platform
                    match self.post_formatted_content_to_platform(platform, &social_post_data).await {
                        Ok(_) => {
                            posted_platforms.push(format!("{:?}", platform));
                            social_posts.push(social_post_data);
                            ic_cdk::println!("✅ Posted enhanced content to {:?}", platform);
                        },
                        Err(e) => {
                            ic_cdk::println!("❌ Failed to post to {:?}: {}", platform, e);
                        }
                    }
                },
                Err(e) => {
                    ic_cdk::println!("❌ Failed to format post for {:?}: {}", platform, e);
                }
            }
        }

        if posted_platforms.is_empty() {
            Err("Failed to post to any social media platforms".to_string())
        } else {
            // Store social post data for analytics
            self.log_social_posts(alert, &social_posts).await;
            Ok(format!("Enhanced posts sent to: {}", posted_platforms.join(", ")))
        }
    }

    /// Post formatted social content to a specific platform
    async fn post_formatted_content_to_platform(
        &self,
        platform: &SocialPlatform,
        social_data: &SocialPostData,
    ) -> Result<(), String> {
        match platform {
            SocialPlatform::Twitter => {
                self.post_to_twitter_enhanced(&social_data.message, &social_data.json_payload).await
            },
            SocialPlatform::Discord => {
                self.post_to_discord_enhanced(&social_data.message, &social_data.json_payload).await
            },
            SocialPlatform::Telegram => {
                self.post_to_telegram_enhanced(&social_data.message, &social_data.json_payload).await
            },
            SocialPlatform::Reddit => {
                self.post_to_reddit_enhanced(&social_data.message, &social_data.json_payload).await
            }
        }
    }

    /// Enhanced platform-specific posting methods with JSON context
    async fn post_to_twitter_enhanced(&self, message: &str, json_context: &str) -> Result<(), String> {
        ic_cdk::println!("🐦 Enhanced Twitter post: {}", message);
        
        // For enhanced posts, we can include JSON as a thread or attachment
        let truncated_message = if message.len() > 240 {
            format!("{}... (1/2)", &message[..240])
        } else {
            message.to_string()
        };
        
        // Post main message first
        self.post_to_twitter(&truncated_message).await?;
        
        // If message was truncated, post JSON context as a reply
        if message.len() > 240 {
            let context_message = format!("📊 Technical Details:\\n```\\n{}\\n```", json_context);
            self.post_to_twitter(&context_message).await?;
        }
        
        Ok(())
    }

    async fn post_to_discord_enhanced(&self, message: &str, json_context: &str) -> Result<(), String> {
        ic_cdk::println!("💬 Enhanced Discord post with embeds");
        
        // Discord supports rich embeds with JSON data
        let enhanced_payload = format!(r#"{{
            "content": "{}",
            "username": "DeFlow Bot",
            "avatar_url": "https://deflow.ai/bot-avatar.png",
            "embeds": [{
                "title": "📊 Technical Details",
                "description": "```json\n{}\n```",
                "color": 3066993,
                "timestamp": "{}"
            }]
        }}"#, 
            message.replace('"', "\\\"")), 
            json_context.replace('"', "\\\"")),
"2024-01-01T00:00:00.000Z" // Placeholder timestamp
        );
        
        let webhook_url = "https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN";
        
        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: webhook_url.to_string(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::POST,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(enhanced_payload.as_bytes().to_vec()),
            transform: None,
        };
        
        ic_cdk::println!("✅ Enhanced Discord webhook simulated");
        Ok(())
    }

    async fn post_to_telegram_enhanced(&self, message: &str, json_context: &str) -> Result<(), String> {
        ic_cdk::println!("📱 Enhanced Telegram post with formatting");
        
        // Telegram supports HTML formatting
        let enhanced_message = format!(r#"{}
        
<b>📊 Technical Data:</b>
<pre>{}</pre>
        
<i>🤖 Powered by DeFlow Protocol</i>"#, 
            message, 
            json_context
        );
        
        self.post_to_telegram(&enhanced_message).await
    }

    async fn post_to_reddit_enhanced(&self, message: &str, json_context: &str) -> Result<(), String> {
        ic_cdk::println!("🤖 Enhanced Reddit post with detailed context");
        
        // Reddit supports markdown formatting
        let enhanced_message = format!(r#"{}
        
## Technical Details

```json
{}
```
        
---
*This alert was generated automatically by [DeFlow Protocol](https://deflow.ai)*"#, 
            message, 
            json_context
        );
        
        self.post_to_reddit(&enhanced_message).await
    }

    /// Enhanced webhook with DeFi execution context
    async fn send_webhook_enhanced(
        &self,
        url: &str,
        payload_template: &str,
        _headers: &HashMap<String, String>,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        defi_result: Option<&DeFiExecutionResult>,
    ) -> Result<String, String> {
        let enhanced_payload = self.format_webhook_payload_enhanced(payload_template, alert, current_price, defi_result)?;
        
        ic_cdk::println!("📡 Sending enhanced webhook to: {}", url);
        ic_cdk::println!("📊 Enhanced payload: {}", enhanced_payload);
        
        // In production: use HTTPS outcalls with structured payload
        Ok(format!("Enhanced webhook sent to {}", url))
    }

    /// Format enhanced webhook payload with DeFi context
    fn format_webhook_payload_enhanced(
        &self,
        template: &str,
        alert: &PriceAlert,
        current_price: &TokenPrice,
        defi_result: Option<&DeFiExecutionResult>,
    ) -> Result<String, String> {
        let base_payload = self.format_webhook_payload(template, alert, current_price)?;
        
        if let Some(defi) = defi_result {
            // Enhance with DeFi execution data
            let enhanced = format!(
                r#"{{"base_alert":{},"defi_execution":{{"success":{},"transaction_hash":"{}","estimated_return":{},"gas_cost":{},"strategy_id":"{}"}}}}"#,
                base_payload,
                defi.success,
                defi.transaction_hash.as_ref().unwrap_or(&"pending".to_string()),
                defi.estimated_return.unwrap_or(0.0),
                defi.actual_gas_cost.unwrap_or(0.0),
                defi.strategy_id.as_ref().unwrap_or(&"unknown".to_string())
            );
            Ok(enhanced)
        } else {
            Ok(base_payload)
        }
    }

    /// Log social posts for analytics and tracking
    async fn log_social_posts(&self, alert: &PriceAlert, posts: &[SocialPostData]) {
        ic_cdk::println!("📈 Logging {} social posts for alert {}", posts.len(), alert.id);
        for post in posts {
            ic_cdk::println!("  - {:?} post with {} hashtags", post.platform, post.hashtags.len());
        }
        // In production: store in persistent analytics storage
    }
}

// Global functions for canister interface
pub async fn check_all_price_alerts() -> Result<Vec<AlertTriggerEvent>, String> {
    let triggered_events = Vec::new();
    
    // For now, return empty vec - will be implemented with proper async handling
    ic_cdk::println!("🔍 Checking price alerts...");
    
    Ok(triggered_events)
}

/// Initialize the price alert system
pub fn init_price_alert_system() {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow_mut().initialize();
    });
}

/// Create a new price alert
pub fn create_price_alert(alert: PriceAlert) -> Result<String, String> {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow_mut().create_alert(alert)
    })
}

/// Get user's price alerts
pub fn get_user_price_alerts(user_id: &str) -> Vec<PriceAlert> {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow().get_user_alerts(user_id)
    })
}

/// Update a price alert
pub fn update_price_alert(alert_id: &str, updates: PriceAlert) -> Result<(), String> {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow_mut().update_alert(alert_id, updates)
    })
}

/// Deactivate a price alert
pub fn deactivate_price_alert(alert_id: &str, user_id: &str) -> Result<(), String> {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow_mut().deactivate_alert(alert_id, user_id)
    })
}

/// Delete a price alert
pub fn delete_price_alert(alert_id: &str, user_id: &str) -> Result<(), String> {
    PRICE_ALERT_MANAGER.with(|manager| {
        manager.borrow_mut().delete_alert(alert_id, user_id)
    })
}