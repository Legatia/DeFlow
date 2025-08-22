# Price Alert & Social Media Integration Feature

## Overview
A comprehensive price monitoring system that allows users to set price targets for tokens, triggering both DeFi actions and automated social media posts for community engagement.

## Feature Components

### 🎯 **Core Functionality**

1. **Price Monitoring Service** - Real-time price tracking with configurable thresholds
2. **Alert Trigger System** - Event-driven architecture for price target hits
3. **DeFi Action Integration** - Execute trades, rebalancing, strategy activation
4. **Social Media Automation** - Post formatted alerts to Twitter/X, Discord, Telegram
5. **Community Dashboard** - Public price alert feed for community engagement

### 🏗️ **System Architecture**

```rust
// Backend Price Alert Service
pub struct PriceAlertService {
    active_alerts: HashMap<String, PriceAlert>,
    price_cache: HashMap<String, TokenPrice>,
    social_integrations: SocialMediaManager,
    defi_executor: DeFiActionExecutor,
}

pub struct PriceAlert {
    id: String,
    user_id: String,
    token_symbol: String,
    target_price: f64,
    condition: PriceCondition, // Above, Below, Change%
    actions: Vec<AlertAction>,
    social_config: Option<SocialPostConfig>,
    created_at: u64,
    expires_at: Option<u64>,
    is_active: bool,
}

pub enum AlertAction {
    DeFiExecution {
        strategy_type: String,
        parameters: HashMap<String, Value>,
    },
    SocialPost {
        platforms: Vec<SocialPlatform>,
        message_template: String,
        include_chart: bool,
    },
    Webhook {
        url: String,
        payload_template: String,
    },
}
```

### 📊 **Price Monitoring Engine**

```rust
// /src/DeFlow_backend/src/defi/price_alert_service.rs
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk_timers;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PriceAlertManager {
    alerts: HashMap<String, PriceAlert>,
    price_sources: Vec<PriceSource>,
    monitoring_interval: u64, // seconds
    last_check: u64,
}

impl PriceAlertManager {
    pub fn new() -> Self {
        Self {
            alerts: HashMap::new(),
            price_sources: vec![
                PriceSource::CoinGecko,
                PriceSource::UniswapV3,
                PriceSource::Binance,
            ],
            monitoring_interval: 30, // Check every 30 seconds
            last_check: 0,
        }
    }

    pub async fn start_monitoring(&mut self) {
        // Set up periodic price checking
        ic_cdk_timers::set_timer_interval(
            std::time::Duration::from_secs(self.monitoring_interval),
            || {
                ic_cdk::spawn(async {
                    let _ = check_all_price_alerts().await;
                });
            }
        );
    }

    pub fn create_alert(&mut self, alert: PriceAlert) -> Result<String, String> {
        // Validate alert parameters
        self.validate_alert(&alert)?;
        
        let alert_id = format!("alert_{}_{}", alert.user_id, ic_cdk::api::time());
        self.alerts.insert(alert_id.clone(), alert);
        
        ic_cdk::println!("Created price alert: {}", alert_id);
        Ok(alert_id)
    }

    async fn check_alert(&self, alert: &PriceAlert) -> Result<bool, String> {
        let current_price = self.get_current_price(&alert.token_symbol).await?;
        
        let condition_met = match alert.condition {
            PriceCondition::Above(target) => current_price > target,
            PriceCondition::Below(target) => current_price < target,
            PriceCondition::PercentChange { base_price, change_percent } => {
                let percent_change = ((current_price - base_price) / base_price) * 100.0;
                percent_change.abs() >= change_percent
            }
        };

        if condition_met {
            self.trigger_alert(alert, current_price).await?;
        }

        Ok(condition_met)
    }

    async fn trigger_alert(&self, alert: &PriceAlert, current_price: f64) -> Result<(), String> {
        ic_cdk::println!("🚨 Price alert triggered for {}: ${}", alert.token_symbol, current_price);
        
        for action in &alert.actions {
            match action {
                AlertAction::DeFiExecution { strategy_type, parameters } => {
                    self.execute_defi_action(alert, strategy_type, parameters).await?;
                }
                AlertAction::SocialPost { platforms, message_template, include_chart } => {
                    self.post_to_social_media(alert, platforms, message_template, current_price, *include_chart).await?;
                }
                AlertAction::Webhook { url, payload_template } => {
                    self.send_webhook(url, payload_template, alert, current_price).await?;
                }
            }
        }

        // Mark alert as triggered (can be configured to auto-disable or repeat)
        Ok(())
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PriceCondition {
    Above(f64),
    Below(f64),
    PercentChange {
        base_price: f64,
        change_percent: f64,
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PriceSource {
    CoinGecko,
    UniswapV3,
    Binance,
    ChainLink,
}
```

### 🤖 **Social Media Integration**

```rust
// /src/DeFlow_backend/src/defi/social_media_manager.rs
use ic_cdk::export::candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SocialMediaManager {
    integrations: HashMap<SocialPlatform, SocialConfig>,
    post_templates: HashMap<String, PostTemplate>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum SocialPlatform {
    Twitter,
    Discord,
    Telegram,
    Reddit,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PostTemplate {
    template: String,
    variables: Vec<String>, // {token}, {price}, {change}, {timestamp}
    hashtags: Vec<String>,
    mentions: Vec<String>,
}

impl SocialMediaManager {
    pub async fn post_price_alert(
        &self, 
        platforms: &[SocialPlatform],
        alert: &PriceAlert,
        current_price: f64
    ) -> Result<Vec<PostResult>, String> {
        let mut results = Vec::new();
        
        for platform in platforms {
            match platform {
                SocialPlatform::Twitter => {
                    let result = self.post_to_twitter(alert, current_price).await?;
                    results.push(result);
                }
                SocialPlatform::Discord => {
                    let result = self.post_to_discord(alert, current_price).await?;
                    results.push(result);
                }
                SocialPlatform::Telegram => {
                    let result = self.post_to_telegram(alert, current_price).await?;
                    results.push(result);
                }
                SocialPlatform::Reddit => {
                    let result = self.post_to_reddit(alert, current_price).await?;
                    results.push(result);
                }
            }
        }
        
        Ok(results)
    }

    async fn post_to_twitter(&self, alert: &PriceAlert, price: f64) -> Result<PostResult, String> {
        let message = self.format_message(alert, price, SocialPlatform::Twitter);
        
        // Twitter API integration via HTTPS outcall
        let twitter_api_url = "https://api.twitter.com/2/tweets";
        let payload = format!(r#"{{
            "text": "{}"
        }}"#, message);
        
        let headers = vec![
            ("Authorization".to_string(), format!("Bearer {}", self.get_twitter_token()?)),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let request = ic_cdk::api::management_canister::http_request::HttpRequest {
            url: twitter_api_url.to_string(),
            method: ic_cdk::api::management_canister::http_request::HttpMethod::POST,
            headers,
            body: Some(payload.as_bytes().to_vec()),
        };

        let response = ic_cdk::api::management_canister::http_request::http_request(request).await
            .map_err(|e| format!("Twitter API call failed: {:?}", e))?;

        Ok(PostResult {
            platform: SocialPlatform::Twitter,
            success: response.status < 300,
            message: String::from_utf8_lossy(&response.body).to_string(),
            post_id: self.extract_post_id(&response.body)?,
        })
    }

    fn format_message(&self, alert: &PriceAlert, price: f64, platform: SocialPlatform) -> String {
        let template = match platform {
            SocialPlatform::Twitter => {
                "🚨 Price Alert: {token} just hit ${price}! 
                
Target: ${target_price} {condition}
Change: {price_change}% in the last hour
Triggered via @DeFlowProtocol

#DeFi #PriceAlert #{token} #Crypto"
            }
            SocialPlatform::Discord => {
                "🚨 **PRICE ALERT TRIGGERED** 🚨

**Token:** {token}
**Current Price:** ${price}
**Target:** ${target_price} {condition}
**24h Change:** {price_change}%
**Triggered:** {timestamp}

*Automated alert via DeFlow Protocol*"
            }
            _ => "Price alert: {token} = ${price}"
        };

        // Replace template variables
        template
            .replace("{token}", &alert.token_symbol)
            .replace("{price}", &format!("{:.4}", price))
            .replace("{target_price}", &format!("{:.4}", self.get_target_price(&alert.condition)))
            .replace("{condition}", &self.format_condition(&alert.condition))
            .replace("{timestamp}", &self.format_timestamp(ic_cdk::api::time()))
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PostResult {
    platform: SocialPlatform,
    success: bool,
    message: String,
    post_id: Option<String>,
}
```

### 🔗 **DeFi Action Integration**

```rust
// Integration with existing DeFi system
impl PriceAlertManager {
    async fn execute_defi_action(
        &self,
        alert: &PriceAlert,
        strategy_type: &str,
        parameters: &HashMap<String, Value>
    ) -> Result<(), String> {
        match strategy_type {
            "buy_order" => {
                let amount = parameters.get("amount")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing amount parameter")?;
                
                self.execute_buy_order(&alert.token_symbol, amount, &alert.user_id).await
            }
            "sell_order" => {
                let amount = parameters.get("amount")
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing amount parameter")?;
                
                self.execute_sell_order(&alert.token_symbol, amount, &alert.user_id).await
            }
            "rebalance_portfolio" => {
                let target_allocation = parameters.get("target_allocation")
                    .and_then(|v| v.as_object())
                    .ok_or("Missing target_allocation parameter")?;
                
                self.rebalance_portfolio(&alert.user_id, target_allocation).await
            }
            "activate_strategy" => {
                let strategy_id = parameters.get("strategy_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing strategy_id parameter")?;
                
                self.activate_strategy(&alert.user_id, strategy_id).await
            }
            _ => Err(format!("Unknown DeFi action: {}", strategy_type))
        }
    }

    async fn execute_buy_order(&self, token: &str, amount: f64, user_id: &str) -> Result<(), String> {
        // Integrate with existing DeFi execution engine
        let strategy_config = StrategyConfig {
            strategy_type: StrategyType::MarketOrder {
                action: OrderAction::Buy,
                token: token.to_string(),
                amount,
            },
            user_id: user_id.to_string(),
            // ... other config
        };

        // Use existing strategy execution system
        crate::defi::strategy_api::execute_strategy(
            user_id.to_string(),
            strategy_config,
            amount
        ).await.map(|_| ())
    }
}
```

### 🎨 **Frontend Components**

```typescript
// /src/DeFlow_frontend/src/components/PriceAlertManager.tsx
import React, { useState, useEffect } from 'react';

interface PriceAlert {
  id: string;
  tokenSymbol: string;
  targetPrice: number;
  condition: 'above' | 'below' | 'change';
  actions: AlertAction[];
  socialConfig?: SocialPostConfig;
  isActive: boolean;
  createdAt: number;
  expiresAt?: number;
}

interface AlertAction {
  type: 'defi' | 'social' | 'webhook';
  config: any;
}

interface SocialPostConfig {
  platforms: ('twitter' | 'discord' | 'telegram')[];
  messageTemplate: string;
  includeChart: boolean;
  hashtags: string[];
}

export const PriceAlertManager: React.FC = () => {
  const [alerts, setAlerts] = useState<PriceAlert[]>([]);
  const [isCreating, setIsCreating] = useState(false);
  
  return (
    <div className="price-alert-manager">
      <div className="alert-header">
        <h2>Price Alerts & Social Automation</h2>
        <button 
          onClick={() => setIsCreating(true)}
          className="btn-primary"
        >
          + Create Alert
        </button>
      </div>

      {/* Alert Creation Form */}
      {isCreating && (
        <PriceAlertForm 
          onSubmit={createAlert}
          onCancel={() => setIsCreating(false)}
        />
      )}

      {/* Active Alerts List */}
      <div className="alerts-list">
        {alerts.map(alert => (
          <PriceAlertCard 
            key={alert.id} 
            alert={alert}
            onEdit={editAlert}
            onDelete={deleteAlert}
            onToggle={toggleAlert}
          />
        ))}
      </div>

      {/* Community Price Feed */}
      <CommunityPriceFeed />
    </div>
  );
};

export const PriceAlertForm: React.FC<{
  onSubmit: (alert: Partial<PriceAlert>) => void;
  onCancel: () => void;
}> = ({ onSubmit, onCancel }) => {
  const [formData, setFormData] = useState<Partial<PriceAlert>>({
    tokenSymbol: '',
    targetPrice: 0,
    condition: 'above',
    actions: [],
  });

  const [socialEnabled, setSocialEnabled] = useState(false);
  const [defiEnabled, setDefiEnabled] = useState(false);

  return (
    <div className="alert-form">
      <h3>Create Price Alert</h3>
      
      {/* Token Selection */}
      <div className="form-group">
        <label>Token</label>
        <TokenSelector 
          value={formData.tokenSymbol}
          onChange={(token) => setFormData(prev => ({ ...prev, tokenSymbol: token }))}
        />
      </div>

      {/* Price Target */}
      <div className="form-group">
        <label>Target Price</label>
        <div className="price-input-group">
          <input
            type="number"
            value={formData.targetPrice}
            onChange={(e) => setFormData(prev => ({ ...prev, targetPrice: parseFloat(e.target.value) }))}
            placeholder="0.00"
            step="0.0001"
          />
          <select 
            value={formData.condition}
            onChange={(e) => setFormData(prev => ({ ...prev, condition: e.target.value as any }))}
          >
            <option value="above">Above</option>
            <option value="below">Below</option>
            <option value="change">% Change</option>
          </select>
        </div>
      </div>

      {/* Action Configuration */}
      <div className="actions-config">
        <h4>Actions to Trigger</h4>
        
        {/* DeFi Actions */}
        <div className="action-group">
          <label>
            <input 
              type="checkbox" 
              checked={defiEnabled}
              onChange={(e) => setDefiEnabled(e.target.checked)}
            />
            Execute DeFi Action
          </label>
          {defiEnabled && (
            <DeFiActionConfig 
              onChange={(config) => updateActionConfig('defi', config)}
            />
          )}
        </div>

        {/* Social Media */}
        <div className="action-group">
          <label>
            <input 
              type="checkbox" 
              checked={socialEnabled}
              onChange={(e) => setSocialEnabled(e.target.checked)}
            />
            Post to Social Media
          </label>
          {socialEnabled && (
            <SocialMediaConfig 
              onChange={(config) => updateActionConfig('social', config)}
            />
          )}
        </div>
      </div>

      {/* Form Actions */}
      <div className="form-actions">
        <button onClick={onCancel} className="btn-secondary">
          Cancel
        </button>
        <button onClick={() => onSubmit(formData)} className="btn-primary">
          Create Alert
        </button>
      </div>
    </div>
  );
};
```

### 📱 **JSON Text Formatting for Social Posts**

```typescript
// /src/DeFlow_frontend/src/utils/socialFormatter.ts
export class SocialPostFormatter {
  static formatPriceAlert(
    token: string,
    currentPrice: number,
    targetPrice: number,
    condition: string,
    platform: 'twitter' | 'discord' | 'telegram'
  ): string {
    const alertData = {
      token,
      current_price: currentPrice,
      target_price: targetPrice,
      condition,
      timestamp: new Date().toISOString(),
      change_24h: this.calculateChange24h(token),
      market_cap: this.getMarketCap(token),
      volume_24h: this.getVolume24h(token),
    };

    switch (platform) {
      case 'twitter':
        return this.formatForTwitter(alertData);
      case 'discord':
        return this.formatForDiscord(alertData);
      case 'telegram':
        return this.formatForTelegram(alertData);
      default:
        return JSON.stringify(alertData, null, 2);
    }
  }

  private static formatForTwitter(data: any): string {
    const emoji = data.change_24h > 0 ? '🚀' : '📉';
    return `${emoji} PRICE ALERT: $${data.token}

💰 Current: $${data.current_price.toFixed(4)}
🎯 Target: $${data.target_price.toFixed(4)} ${data.condition}
📊 24h: ${data.change_24h > 0 ? '+' : ''}${data.change_24h.toFixed(2)}%

#DeFi #${data.token} #PriceAlert #DeFlow`;
  }

  private static formatForDiscord(data: any): string {
    return `🚨 **PRICE ALERT TRIGGERED**

\`\`\`json
${JSON.stringify(data, null, 2)}
\`\`\`

**Token:** ${data.token}
**Price:** $${data.current_price}
**Target:** $${data.target_price} ${data.condition}
**24h Change:** ${data.change_24h}%

*Powered by DeFlow Protocol*`;
  }
}
```

## Implementation Plan

### Phase 1: Core Backend Infrastructure
1. **Price Monitoring Service** - Real-time price tracking
2. **Alert Management System** - CRUD operations for price alerts  
3. **Trigger Engine** - Event-driven alert processing
4. **Basic DeFi Integration** - Execute simple buy/sell orders

### Phase 2: Social Media Integration  
1. **Twitter API Integration** - Automated tweet posting
2. **Discord Webhook** - Channel notifications  
3. **Message Formatting** - JSON to social media text conversion
4. **Template System** - Customizable post templates

### Phase 3: Advanced Features
1. **Community Dashboard** - Public price alert feed
2. **Alert Analytics** - Performance tracking and insights
3. **Advanced DeFi Actions** - Strategy activation, portfolio rebalancing
4. **Mobile Push Notifications** - Real-time alerts

### Phase 4: Optimization & Scaling
1. **Rate Limiting** - Prevent API abuse
2. **Batch Processing** - Efficient alert checking
3. **Caching Layer** - Price data optimization
4. **Circuit Breakers** - Resilient external API calls

## Benefits for DeFlow Community

1. **Increased Engagement** - Automated social media presence
2. **Community Building** - Shared price alerts and discussions  
3. **Educational Value** - Real-time market insights
4. **User Retention** - Interactive and valuable features
5. **Viral Growth** - Social media sharing drives new users

This feature perfectly combines DeFlow's DeFi automation capabilities with community engagement, creating a powerful tool for both individual users and the broader crypto community!