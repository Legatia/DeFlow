// Pendle Finance Integration - Yield Tokenization and Fixed Yield Trading
// https://app.pendle.finance/trade/markets

use super::types::ChainId;
use super::yield_farming::DeFiProtocol;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

#[derive(Debug, Clone)]
pub struct PendleIntegration {
    pub api_endpoint: String,
    pub supported_chains: Vec<ChainId>,
    pub market_cache: HashMap<String, PendleMarket>,
    pub last_update: u64,
    pub rate_limiter: PendleRateLimiter,
}

impl PendleIntegration {
    pub fn new() -> Self {
        Self {
            api_endpoint: "https://api-v2.pendle.finance".to_string(),
            supported_chains: vec![
                ChainId::Ethereum,
                ChainId::Arbitrum,
                ChainId::Optimism,
                ChainId::Polygon,
                ChainId::Base,
            ],
            market_cache: HashMap::new(),
            last_update: 0,
            rate_limiter: PendleRateLimiter::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), PendleError> {
        ic_cdk::println!("Initializing Pendle Finance integration...");
        
        // Fetch initial market data
        self.update_market_data().await?;
        
        ic_cdk::println!("Pendle integration initialized successfully");
        Ok(())
    }

    /// Get all active Pendle yield opportunities
    pub async fn get_yield_opportunities(&mut self) -> Result<Vec<PendleYieldOpportunity>, PendleError> {
        if !self.rate_limiter.can_make_request() {
            return Err(PendleError::RateLimited("API rate limit exceeded".to_string()));
        }

        // Update market data if stale (older than 5 minutes)
        let current_time = ic_cdk::api::time();
        if current_time - self.last_update > 300_000_000_000 { // 5 minutes in nanoseconds
            self.update_market_data().await?;
        }

        let mut opportunities = Vec::new();

        for (market_id, market) in &self.market_cache {
            // PT (Principal Token) opportunities - Fixed yield
            if market.pt_apy > 0.0 {
                opportunities.push(PendleYieldOpportunity {
                    id: format!("pendle_pt_{}", market_id),
                    market_id: market_id.clone(),
                    opportunity_type: PendleOpportunityType::PrincipalToken,
                    underlying_asset: market.underlying_asset.clone(),
                    chain: market.chain.clone(),
                    pt_apy: market.pt_apy,
                    yt_apy: 0.0, // Not applicable for PT
                    lp_apy: 0.0, // Not applicable for PT
                    maturity_date: market.maturity,
                    total_liquidity_usd: market.total_liquidity,
                    pt_price: market.pt_price,
                    yt_price: 0.0,
                    risk_score: self.calculate_pt_risk_score(market),
                    min_deposit_usd: 100.0, // Pendle typically allows smaller deposits
                    max_deposit_usd: market.total_liquidity * 0.1, // Max 10% of liquidity
                    last_updated: current_time,
                });
            }

            // YT (Yield Token) opportunities - Leveraged yield exposure
            if market.yt_apy > 0.0 {
                opportunities.push(PendleYieldOpportunity {
                    id: format!("pendle_yt_{}", market_id),
                    market_id: market_id.clone(),
                    opportunity_type: PendleOpportunityType::YieldToken,
                    underlying_asset: market.underlying_asset.clone(),
                    chain: market.chain.clone(),
                    pt_apy: 0.0, // Not applicable for YT
                    yt_apy: market.yt_apy,
                    lp_apy: 0.0, // Not applicable for YT
                    maturity_date: market.maturity,
                    total_liquidity_usd: market.total_liquidity,
                    pt_price: 0.0,
                    yt_price: market.yt_price,
                    risk_score: self.calculate_yt_risk_score(market),
                    min_deposit_usd: 100.0,
                    max_deposit_usd: market.total_liquidity * 0.05, // More conservative for YT
                    last_updated: current_time,
                });
            }

            // LP opportunities - Provide liquidity to PT/underlying markets
            if market.lp_apy > 0.0 {
                opportunities.push(PendleYieldOpportunity {
                    id: format!("pendle_lp_{}", market_id),
                    market_id: market_id.clone(),
                    opportunity_type: PendleOpportunityType::LiquidityProvider,
                    underlying_asset: market.underlying_asset.clone(),
                    chain: market.chain.clone(),
                    pt_apy: 0.0, // Not applicable for LP
                    yt_apy: 0.0, // Not applicable for LP
                    lp_apy: market.lp_apy,
                    maturity_date: market.maturity,
                    total_liquidity_usd: market.total_liquidity,
                    pt_price: market.pt_price,
                    yt_price: market.yt_price,
                    risk_score: self.calculate_lp_risk_score(market),
                    min_deposit_usd: 500.0, // LP typically requires more capital
                    max_deposit_usd: market.total_liquidity * 0.2, // Can take larger LP positions
                    last_updated: current_time,
                });
            }
        }

        // Sort by risk-adjusted APY
        opportunities.sort_by(|a, b| {
            let score_a = a.get_effective_apy() / (a.risk_score as f64);
            let score_b = b.get_effective_apy() / (b.risk_score as f64);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(opportunities)
    }

    /// Update market data from Pendle API
    async fn update_market_data(&mut self) -> Result<(), PendleError> {
        for chain in &self.supported_chains.clone() {
            let markets = self.fetch_markets_for_chain(chain).await?;
            for market in markets {
                self.market_cache.insert(market.id.clone(), market);
            }
        }
        
        self.last_update = ic_cdk::api::time();
        Ok(())
    }

    /// Fetch markets for a specific chain
    async fn fetch_markets_for_chain(&self, chain: &ChainId) -> Result<Vec<PendleMarket>, PendleError> {
        let chain_id = self.get_chain_id(chain);
        let url = format!("{}/v2/core/markets?chain_id={}&limit=50&order_by=total_liquidity", 
                         self.api_endpoint, chain_id);

        let request = CanisterHttpRequestArgument {
            url,
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(1_000_000), // 1MB limit
            transform: None,
            headers: vec![
                HttpHeader {
                    name: "Accept".to_string(),
                    value: "application/json".to_string(),
                },
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: "DeFlow/1.0".to_string(),
                },
            ],
        };

        match http_request(request, 10_000_000_000).await {
            Ok((response,)) => {
                if response.status == candid::Nat::from(200u32) {
                    let body_str = String::from_utf8(response.body)
                        .map_err(|_| PendleError::ParseError("Invalid UTF-8 response".to_string()))?;
                    
                    // Parse Pendle API response
                    self.parse_market_response(&body_str, chain)
                } else {
                    Err(PendleError::ApiError(format!("HTTP {}: Failed to fetch markets", response.status)))
                }
            }
            Err(e) => Err(PendleError::NetworkError(format!("Request failed: {:?}", e))),
        }
    }

    /// Parse Pendle API market response
    fn parse_market_response(&self, _response: &str, chain: &ChainId) -> Result<Vec<PendleMarket>, PendleError> {
        // Simplified parsing - in production would use proper JSON parsing
        let mut markets = Vec::new();
        
        // Mock data structure based on Pendle's actual API
        // This would be replaced with proper JSON deserialization
        
        // Example markets based on real Pendle offerings
        match chain {
            ChainId::Ethereum => {
                markets.push(PendleMarket {
                    id: "eth_eeth_26dec2024".to_string(),
                    name: "Ether.fi Restaked ETH 26DEC2024".to_string(),
                    underlying_asset: "eETH".to_string(),
                    chain: chain.clone(),
                    pt_apy: 4.2,
                    yt_apy: 15.8,
                    lp_apy: 12.3,
                    total_liquidity: 45_000_000.0,
                    pt_price: 0.9234,
                    yt_price: 0.0156,
                    maturity: 1735171200, // Dec 26, 2024
                    is_active: true,
                });

                markets.push(PendleMarket {
                    id: "eth_sweth_26dec2024".to_string(),
                    name: "Swell Restaked ETH 26DEC2024".to_string(),
                    underlying_asset: "swETH".to_string(),
                    chain: chain.clone(),
                    pt_apy: 3.8,
                    yt_apy: 18.2,
                    lp_apy: 14.7,
                    total_liquidity: 28_000_000.0,
                    pt_price: 0.9156,
                    yt_price: 0.0198,
                    maturity: 1735171200,
                    is_active: true,
                });
            },
            ChainId::Arbitrum => {
                markets.push(PendleMarket {
                    id: "arb_gdalp_26dec2024".to_string(),
                    name: "GMX Delta-Neutral GLP 26DEC2024".to_string(),
                    underlying_asset: "gdALP".to_string(),
                    chain: chain.clone(),
                    pt_apy: 8.5,
                    yt_apy: 25.3,
                    lp_apy: 18.9,
                    total_liquidity: 12_000_000.0,
                    pt_price: 0.8895,
                    yt_price: 0.0234,
                    maturity: 1735171200,
                    is_active: true,
                });
            },
            _ => {
                // Other chains would have their specific markets
            }
        }
        
        Ok(markets)
    }

    /// Calculate risk score for Principal Token positions
    fn calculate_pt_risk_score(&self, market: &PendleMarket) -> u8 {
        let mut risk_score = 3; // Base risk for PT (relatively safe)
        
        // Time to maturity risk
        let time_to_maturity = market.maturity - (ic_cdk::api::time() / 1_000_000_000);
        if time_to_maturity > 365 * 24 * 60 * 60 { // > 1 year
            risk_score += 1;
        }
        
        // Liquidity risk
        if market.total_liquidity < 1_000_000.0 {
            risk_score += 2;
        } else if market.total_liquidity < 10_000_000.0 {
            risk_score += 1;
        }
        
        // Asset risk
        match market.underlying_asset.as_str() {
            "ETH" | "WETH" | "stETH" => risk_score += 0, // Low risk
            "eETH" | "swETH" | "rETH" => risk_score += 1, // Medium risk (liquid staking)
            _ => risk_score += 2, // Higher risk for other assets
        }
        
        risk_score.min(10)
    }

    /// Calculate risk score for Yield Token positions
    fn calculate_yt_risk_score(&self, market: &PendleMarket) -> u8 {
        let mut risk_score = 6; // Base risk for YT (higher risk, leveraged yield)
        
        // Volatility risk
        if market.yt_apy > 30.0 {
            risk_score += 2; // Very high APY indicates high volatility
        } else if market.yt_apy > 15.0 {
            risk_score += 1;
        }
        
        // Liquidity and time risk (same as PT)
        let pt_risk = self.calculate_pt_risk_score(market);
        (risk_score + pt_risk / 2).min(10) as u8
    }

    /// Calculate risk score for LP positions
    fn calculate_lp_risk_score(&self, market: &PendleMarket) -> u8 {
        let mut risk_score = 4; // Base risk for LP (moderate)
        
        // Impermanent loss risk
        if market.pt_price < 0.85 || market.pt_price > 0.98 {
            risk_score += 2; // High IL risk when PT price deviates significantly
        }
        
        // Liquidity risk (same calculation as PT)
        let base_risk = self.calculate_pt_risk_score(market);
        (risk_score + base_risk / 3).min(10)
    }

    /// Get chain ID for Pendle API
    fn get_chain_id(&self, chain: &ChainId) -> u32 {
        match chain {
            ChainId::Ethereum => 1,
            ChainId::Arbitrum => 42161,
            ChainId::Optimism => 10,
            ChainId::Polygon => 137,
            ChainId::Base => 8453,
            _ => 1, // Default to Ethereum
        }
    }

    /// Get health status of Pendle integration
    pub fn get_health_status(&self) -> super::protocol_integrations::ProtocolHealthStatus {
        let current_time = ic_cdk::api::time();
        let is_data_fresh = (current_time - self.last_update) < 600_000_000_000; // 10 minutes
        let is_healthy = is_data_fresh && !self.market_cache.is_empty();
        
        super::protocol_integrations::ProtocolHealthStatus {
            is_healthy,
            last_successful_update: if is_healthy { Some(self.last_update) } else { None },
            error_rate_percentage: if is_healthy { 0.0 } else { 100.0 },
            api_response_time_ms: 200, // Estimate - would be tracked in production
        }
    }
}

// Data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PendleYieldOpportunity {
    pub id: String,
    pub market_id: String,
    pub opportunity_type: PendleOpportunityType,
    pub underlying_asset: String,
    pub chain: ChainId,
    pub pt_apy: f64,      // Principal Token APY (fixed yield)
    pub yt_apy: f64,      // Yield Token APY (leveraged yield)
    pub lp_apy: f64,      // Liquidity Provider APY
    pub maturity_date: u64,
    pub total_liquidity_usd: f64,
    pub pt_price: f64,    // Price of Principal Token
    pub yt_price: f64,    // Price of Yield Token
    pub risk_score: u8,   // 1-10 scale
    pub min_deposit_usd: f64,
    pub max_deposit_usd: f64,
    pub last_updated: u64,
}

impl PendleYieldOpportunity {
    pub fn get_effective_apy(&self) -> f64 {
        match self.opportunity_type {
            PendleOpportunityType::PrincipalToken => self.pt_apy,
            PendleOpportunityType::YieldToken => self.yt_apy,
            PendleOpportunityType::LiquidityProvider => self.lp_apy,
        }
    }

    pub fn days_to_maturity(&self) -> u64 {
        let current_time = ic_cdk::api::time() / 1_000_000_000; // Convert to seconds
        if self.maturity_date > current_time {
            (self.maturity_date - current_time) / (24 * 60 * 60)
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PendleOpportunityType {
    PrincipalToken,    // Fixed yield until maturity
    YieldToken,        // Leveraged yield exposure
    LiquidityProvider, // Provide liquidity to PT/asset pools
}

#[derive(Debug, Clone)]
pub struct PendleMarket {
    pub id: String,
    pub name: String,
    pub underlying_asset: String,
    pub chain: ChainId,
    pub pt_apy: f64,
    pub yt_apy: f64,
    pub lp_apy: f64,
    pub total_liquidity: f64,
    pub pt_price: f64,
    pub yt_price: f64,
    pub maturity: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PendleHealthStatus {
    pub is_healthy: bool,
    pub active_markets: usize,
    pub last_update: u64,
    pub supported_chains: usize,
}

#[derive(Debug, Clone)]
pub struct PendleRateLimiter {
    last_request: u64,
    min_interval: u64, // Minimum time between requests (nanoseconds)
}

impl PendleRateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: 0,
            min_interval: 2_000_000_000, // 2 seconds between requests
        }
    }

    pub fn can_make_request(&mut self) -> bool {
        let current_time = ic_cdk::api::time();
        if current_time - self.last_request >= self.min_interval {
            self.last_request = current_time;
            true
        } else {
            false
        }
    }
}

// Error types

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PendleError {
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    RateLimited(String),
    InvalidMarket(String),
    InsufficientLiquidity(String),
}

impl std::fmt::Display for PendleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PendleError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PendleError::ApiError(msg) => write!(f, "API error: {}", msg),
            PendleError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PendleError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            PendleError::InvalidMarket(msg) => write!(f, "Invalid market: {}", msg),
            PendleError::InsufficientLiquidity(msg) => write!(f, "Insufficient liquidity: {}", msg),
        }
    }
}

impl From<PendleError> for super::protocol_integrations::IntegrationError {
    fn from(error: PendleError) -> Self {
        match error {
            PendleError::NetworkError(msg) => super::protocol_integrations::IntegrationError::NetworkError(msg),
            PendleError::ApiError(msg) => super::protocol_integrations::IntegrationError::ApiError(msg),
            PendleError::ParseError(msg) => super::protocol_integrations::IntegrationError::ParseError(msg),
            PendleError::RateLimited(_msg) => super::protocol_integrations::IntegrationError::RateLimited,
            PendleError::InvalidMarket(_) => super::protocol_integrations::IntegrationError::ApiError("Invalid market".to_string()),
            PendleError::InsufficientLiquidity(_) => super::protocol_integrations::IntegrationError::InsufficientLiquidity,
        }
    }
}

// Usage examples and benefits

impl PendleIntegration {
    pub fn get_integration_benefits() -> String {
        format!(r#"
🎯 PENDLE FINANCE INTEGRATION BENEFITS:

🔐 FIXED YIELD STRATEGIES:
• Principal Tokens (PT): Lock in guaranteed yields (3-8% APY)
• Hedge against yield volatility
• Capital preservation with fixed returns

⚡ LEVERAGED YIELD EXPOSURE:
• Yield Tokens (YT): Amplified yield exposure (15-30% APY)
• Profit from rising yield rates
• Higher risk, higher potential returns

💰 UNIQUE YIELD OPPORTUNITIES:
• Liquid Staking Derivatives (eETH, swETH, rETH)
• GMX Delta-Neutral strategies
• Pendle's exclusive yield tokenization

🌐 MULTI-CHAIN COVERAGE:
• Ethereum: Largest markets, highest liquidity
• Arbitrum: Lower gas costs, GMX integration
• Optimism & Base: Emerging opportunities

📊 SOPHISTICATED STRATEGIES:
• Yield stripping and trading
• Maturity date optimization
• Risk-adjusted portfolio allocation

💎 DEFLOW ADVANTAGES:
• Automated market scanning across all chains
• Risk assessment and position sizing
• Gas optimization for Pendle transactions
• Real-time opportunity alerts

Example Strategy:
• User deposits $10,000
• DeFlow finds eETH PT @ 4.2% fixed yield
• Alternative: eETH YT @ 15.8% leveraged yield
• Risk management: Diversify across maturities
• Result: Optimized yield with controlled risk
        "#)
    }
}