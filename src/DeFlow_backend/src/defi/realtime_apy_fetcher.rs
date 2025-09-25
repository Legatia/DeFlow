// Real-time APY data fetcher using HTTP outcalls
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
    TransformContext, TransformArgs
};
use candid::{CandidType, Deserialize};
use serde_json;
use std::collections::HashMap;
use ic_cdk_timers::{set_timer_interval, TimerId};
use std::time::Duration;
use std::cell::RefCell;

// APY data structure for caching
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct RealtimeAPYData {
    pub protocol: String,
    pub chain: String,
    pub token: String,
    pub apy: f64,
    pub tvl_usd: u64,
    pub risk_score: f64,
    pub last_updated: u64,
    pub source: String, // DeFiLlama, protocol API, etc.
}

// Gas fee data structure
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct GasFeeData {
    pub chain: String,
    pub slow_gwei: f64,
    pub standard_gwei: f64,
    pub fast_gwei: f64,
    pub eth_price_usd: f64,
    pub last_updated: u64,
}

// Gas monitoring opportunity
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct GasMonitoringOpportunity {
    pub apy_data: RealtimeAPYData,
    pub investment_amount_usd: f64,
    pub monitoring_started: u64,
    pub target_gas_cost_usd: f64, // Max acceptable gas cost
    pub target_payback_days: f64, // Days to break even on gas fees
}

// Protocol switching tracking
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct ProtocolSwitchRecord {
    pub user_id: String,
    pub last_switch_timestamp: u64,
    pub switches_today: u32,
    pub current_protocol: String,
    pub current_chain: String,
}

// Thread-safe storage for APY data and switch records
thread_local! {
    static APY_CACHE: RefCell<HashMap<String, RealtimeAPYData>> = RefCell::new(HashMap::new());
    static SWITCH_RECORDS: RefCell<HashMap<String, ProtocolSwitchRecord>> = RefCell::new(HashMap::new());
    static FETCHER_TIMER: RefCell<Option<TimerId>> = RefCell::new(None);
    static GAS_FEE_CACHE: RefCell<HashMap<String, GasFeeData>> = RefCell::new(HashMap::new());
    static GAS_MONITORING_QUEUE: RefCell<Vec<GasMonitoringOpportunity>> = RefCell::new(Vec::new());
    static GAS_MONITOR_TIMER: RefCell<Option<TimerId>> = RefCell::new(None);
}

pub struct RealtimeAPYFetcher {
    pub fetch_interval_seconds: u64,
    pub max_switches_per_day: u32,
    pub min_apy_improvement_threshold: f64, // Minimum % improvement to justify switch
}

impl Default for RealtimeAPYFetcher {
    fn default() -> Self {
        Self {
            fetch_interval_seconds: 1800, // 30 minutes
            max_switches_per_day: 1,      // Once per day
            min_apy_improvement_threshold: 0.5, // 0.5% minimum improvement
        }
    }
}

impl RealtimeAPYFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    // Start the periodic APY fetching timer
    pub fn start_periodic_fetch(&self) {
        let interval = Duration::from_secs(self.fetch_interval_seconds);

        let timer_id = set_timer_interval(interval, || {
            ic_cdk::spawn(async {
                if let Err(e) = Self::fetch_all_apy_data().await {
                    ic_cdk::println!("Error fetching APY data: {}", e);
                }
            });
        });

        FETCHER_TIMER.with(|timer| {
            *timer.borrow_mut() = Some(timer_id);
        });

        ic_cdk::println!("Started APY fetcher with {}s intervals", self.fetch_interval_seconds);
    }

    // Fetch APY data from multiple sources
    async fn fetch_all_apy_data() -> Result<(), String> {
        // Fetch from DeFiLlama
        Self::fetch_defillama_yields().await?;

        // Fetch from Aave API
        Self::fetch_aave_rates().await?;

        // Fetch from Compound API
        Self::fetch_compound_rates().await?;

        ic_cdk::println!("Successfully updated APY data from all sources");
        Ok(())
    }

    // Fetch yield data from DeFiLlama API
    async fn fetch_defillama_yields() -> Result<(), String> {
        let url = "https://yields.llama.fi/pools";

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(2_000_000), // 2MB max
            transform: Some(TransformContext::from_name("transform_defillama_response".to_string(), vec![])),
            headers: vec![
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: "DeFlow-ICP-Canister/1.0".to_string(),
                },
                HttpHeader {
                    name: "Accept".to_string(),
                    value: "application/json".to_string(),
                },
            ],
        };

        match http_request(request, 25_000_000_000).await {
            Ok((response,)) => {
                Self::parse_defillama_response(response.body)?;
                Ok(())
            }
            Err((r, m)) => {
                Err(format!("HTTP request failed: {:?} - {}", r, m))
            }
        }
    }

    // Parse DeFiLlama response and cache APY data
    fn parse_defillama_response(body: Vec<u8>) -> Result<(), String> {
        let response_str = String::from_utf8(body)
            .map_err(|e| format!("Failed to parse response as UTF-8: {}", e))?;

        let json_data: serde_json::Value = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Some(pools) = json_data["data"].as_array() {
            let current_time = ic_cdk::api::time();
            let mut cached_count = 0;

            for pool in pools.iter().take(100) { // Limit to top 100 pools
                if let (Some(protocol), Some(chain), Some(symbol), Some(apy), Some(tvl)) = (
                    pool["project"].as_str(),
                    pool["chain"].as_str(),
                    pool["symbol"].as_str(),
                    pool["apy"].as_f64(),
                    pool["tvlUsd"].as_f64(),
                ) {
                    // Filter for major protocols and chains we support
                    if Self::is_supported_protocol(protocol) && Self::is_supported_chain(chain) {
                        let cache_key = format!("{}:{}:{}", protocol, chain, symbol);

                        let apy_data = RealtimeAPYData {
                            protocol: protocol.to_string(),
                            chain: chain.to_string(),
                            token: symbol.to_string(),
                            apy,
                            tvl_usd: tvl as u64,
                            risk_score: Self::calculate_risk_score(protocol, tvl),
                            last_updated: current_time,
                            source: "DeFiLlama".to_string(),
                        };

                        APY_CACHE.with(|cache| {
                            cache.borrow_mut().insert(cache_key, apy_data);
                        });

                        cached_count += 1;
                    }
                }
            }

            ic_cdk::println!("Cached {} APY records from DeFiLlama", cached_count);
            Ok(())
        } else {
            Err("Invalid DeFiLlama response format".to_string())
        }
    }

    // Fetch Aave lending rates
    async fn fetch_aave_rates() -> Result<(), String> {
        // Aave V3 subgraph endpoints for different chains
        let endpoints = vec![
            ("Ethereum", "https://api.thegraph.com/subgraphs/name/aave/protocol-v3"),
            ("Arbitrum", "https://api.thegraph.com/subgraphs/name/aave/protocol-v3-arbitrum"),
            ("Polygon", "https://api.thegraph.com/subgraphs/name/aave/protocol-v3-polygon"),
        ];

        for (chain, url) in endpoints {
            let query = r#"
            {
              reserves(first: 10, orderBy: totalLiquidity, orderDirection: desc) {
                id
                symbol
                liquidityRate
                variableBorrowRate
                totalLiquidity
                liquidityIndex
              }
            }
            "#;

            let request_body = serde_json::json!({
                "query": query
            });

            let request = CanisterHttpRequestArgument {
                url: url.to_string(),
                method: HttpMethod::POST,
                body: Some(request_body.to_string().into_bytes()),
                max_response_bytes: Some(1_000_000),
                transform: Some(TransformContext::from_name("transform_aave_response".to_string(), vec![])),
                headers: vec![
                    HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    },
                ],
            };

            match http_request(request, 25_000_000_000).await {
                Ok((response,)) => {
                    Self::parse_aave_response(response.body, chain)?;
                }
                Err((r, m)) => {
                    ic_cdk::println!("Aave request failed for {}: {:?} - {}", chain, r, m);
                }
            }
        }

        Ok(())
    }

    fn parse_aave_response(body: Vec<u8>, chain: &str) -> Result<(), String> {
        let response_str = String::from_utf8(body)
            .map_err(|e| format!("Failed to parse response as UTF-8: {}", e))?;

        let json_data: serde_json::Value = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Some(reserves) = json_data["data"]["reserves"].as_array() {
            let current_time = ic_cdk::api::time();

            for reserve in reserves {
                if let (Some(symbol), Some(liquidity_rate), Some(total_liquidity)) = (
                    reserve["symbol"].as_str(),
                    reserve["liquidityRate"].as_str(),
                    reserve["totalLiquidity"].as_str(),
                ) {
                    // Convert from ray (27 decimals) to percentage
                    if let (Ok(rate), Ok(liquidity)) = (
                        liquidity_rate.parse::<f64>(),
                        total_liquidity.parse::<f64>(),
                    ) {
                        let apy = (rate / 1e27) * 100.0; // Convert ray to percentage

                        if apy > 0.1 { // Only cache if APY > 0.1%
                            let cache_key = format!("Aave:{}:{}", chain, symbol);

                            let apy_data = RealtimeAPYData {
                                protocol: "Aave".to_string(),
                                chain: chain.to_string(),
                                token: symbol.to_string(),
                                apy,
                                tvl_usd: (liquidity / 1e18) as u64, // Convert from wei
                                risk_score: 2.0, // Aave is low risk
                                last_updated: current_time,
                                source: "Aave API".to_string(),
                            };

                            APY_CACHE.with(|cache| {
                                cache.borrow_mut().insert(cache_key, apy_data);
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Fetch Compound rates
    async fn fetch_compound_rates() -> Result<(), String> {
        let url = "https://api.compound.finance/api/v2/ctoken";

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(1_000_000),
            transform: Some(TransformContext::from_name("transform_compound_response".to_string(), vec![])),
            headers: vec![
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: "DeFlow-ICP-Canister/1.0".to_string(),
                },
            ],
        };

        match http_request(request, 25_000_000_000).await {
            Ok((response,)) => {
                Self::parse_compound_response(response.body)?;
                Ok(())
            }
            Err((r, m)) => {
                Err(format!("Compound HTTP request failed: {:?} - {}", r, m))
            }
        }
    }

    fn parse_compound_response(body: Vec<u8>) -> Result<(), String> {
        let response_str = String::from_utf8(body)
            .map_err(|e| format!("Failed to parse response as UTF-8: {}", e))?;

        let json_data: serde_json::Value = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if let Some(ctokens) = json_data["cToken"].as_array() {
            let current_time = ic_cdk::api::time();

            for ctoken in ctokens {
                if let (Some(symbol), Some(supply_rate), Some(total_supply)) = (
                    ctoken["underlying_symbol"].as_str(),
                    ctoken["supply_rate"]["value"].as_str(),
                    ctoken["total_supply"]["value"].as_str(),
                ) {
                    if let (Ok(rate), Ok(supply)) = (
                        supply_rate.parse::<f64>(),
                        total_supply.parse::<f64>(),
                    ) {
                        let apy = rate * 100.0; // Convert to percentage

                        if apy > 0.1 {
                            let cache_key = format!("Compound:Ethereum:{}", symbol);

                            let apy_data = RealtimeAPYData {
                                protocol: "Compound".to_string(),
                                chain: "Ethereum".to_string(),
                                token: symbol.to_string(),
                                apy,
                                tvl_usd: supply as u64,
                                risk_score: 2.5, // Compound is low-medium risk
                                last_updated: current_time,
                                source: "Compound API".to_string(),
                            };

                            APY_CACHE.with(|cache| {
                                cache.borrow_mut().insert(cache_key, apy_data);
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Check if protocol switching is allowed for user
    pub fn can_switch_protocol(user_id: &str) -> bool {
        SWITCH_RECORDS.with(|records| {
            let records = records.borrow();
            if let Some(record) = records.get(user_id) {
                let current_time = ic_cdk::api::time();
                let one_day_nanos = 24 * 60 * 60 * 1_000_000_000; // 24 hours in nanoseconds

                // Check if last switch was more than 24 hours ago
                current_time - record.last_switch_timestamp > one_day_nanos
            } else {
                true // First time, allow switch
            }
        })
    }

    // Record a protocol switch for rate limiting
    pub fn record_protocol_switch(user_id: &str, new_protocol: &str, new_chain: &str) {
        let current_time = ic_cdk::api::time();

        SWITCH_RECORDS.with(|records| {
            let mut records = records.borrow_mut();

            let record = ProtocolSwitchRecord {
                user_id: user_id.to_string(),
                last_switch_timestamp: current_time,
                switches_today: 1,
                current_protocol: new_protocol.to_string(),
                current_chain: new_chain.to_string(),
            };

            records.insert(user_id.to_string(), record);
        });

        ic_cdk::println!("Recorded protocol switch for user {} to {} on {}", user_id, new_protocol, new_chain);
    }

    // Get current best APY for a token across all protocols/chains
    pub fn get_best_apy_opportunity(token: &str, trading_style: &super::ethereum::TradingStyle) -> Option<RealtimeAPYData> {
        APY_CACHE.with(|cache| {
            let cache = cache.borrow();
            let style_params = trading_style.get_params();

            cache.values()
                .filter(|data| {
                    data.token.to_uppercase() == token.to_uppercase() &&
                    data.apy >= style_params.min_apy &&
                    Self::is_chain_preferred(&data.chain, &style_params.preferred_chains) &&
                    data.risk_score <= Self::get_max_risk_for_style(trading_style)
                })
                .max_by(|a, b| {
                    // Calculate score incorporating APY, gas costs, and risk
                    let score_a = Self::calculate_opportunity_score(a, &style_params);
                    let score_b = Self::calculate_opportunity_score(b, &style_params);
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned()
        })
    }

    // Get all cached APY data
    pub fn get_all_cached_data() -> Vec<RealtimeAPYData> {
        APY_CACHE.with(|cache| {
            cache.borrow().values().cloned().collect()
        })
    }

    // Get conservative fallback data when APIs fail
    pub fn get_conservative_fallback_data() -> Vec<RealtimeAPYData> {
        let current_time = ic_cdk::api::time();

        vec![
            RealtimeAPYData {
                protocol: "Aave".to_string(),
                chain: "Arbitrum".to_string(),
                token: "USDC".to_string(),
                apy: 2.5, // Ultra-conservative estimate
                tvl_usd: 1_000_000_000,
                risk_score: 2.0,
                last_updated: current_time,
                source: "Conservative Fallback".to_string(),
            },
            RealtimeAPYData {
                protocol: "Aave".to_string(),
                chain: "Base".to_string(),
                token: "USDC".to_string(),
                apy: 2.3,
                tvl_usd: 500_000_000,
                risk_score: 2.0,
                last_updated: current_time,
                source: "Conservative Fallback".to_string(),
            },
        ]
    }

    // Check if cached data is stale (older than 2 hours)
    pub fn is_data_stale() -> bool {
        APY_CACHE.with(|cache| {
            let cache = cache.borrow();
            if cache.is_empty() {
                return true;
            }

            let current_time = ic_cdk::api::time();
            let two_hours_nanos = 2 * 60 * 60 * 1_000_000_000; // 2 hours in nanoseconds

            cache.values().all(|data| {
                current_time - data.last_updated > two_hours_nanos
            })
        })
    }

    // Helper functions
    fn is_supported_protocol(protocol: &str) -> bool {
        matches!(protocol.to_lowercase().as_str(),
            "aave" | "compound" | "uniswap" | "curve" | "yearn" | "convex" | "lido" | "rocket-pool"
        )
    }

    fn is_supported_chain(chain: &str) -> bool {
        matches!(chain.to_lowercase().as_str(),
            "ethereum" | "arbitrum" | "optimism" | "polygon" | "base" | "avalanche"
        )
    }

    fn calculate_risk_score(protocol: &str, tvl: f64) -> f64 {
        let base_risk: f64 = match protocol.to_lowercase().as_str() {
            "aave" | "compound" => 2.0,
            "uniswap" | "curve" => 4.0,
            "yearn" | "convex" => 5.0,
            _ => 6.0,
        };

        // Adjust risk based on TVL (higher TVL = lower risk)
        let tvl_factor: f64 = if tvl > 1_000_000_000.0 { -0.5 } else if tvl > 100_000_000.0 { 0.0 } else { 1.0 };

        (base_risk + tvl_factor).max(1.0).min(10.0)
    }

    fn is_chain_preferred(chain: &str, preferred_chains: &[super::ethereum::EvmChain]) -> bool {
        preferred_chains.iter().any(|preferred| preferred.name() == chain)
    }

    fn get_max_risk_for_style(trading_style: &super::ethereum::TradingStyle) -> f64 {
        use super::ethereum::TradingStyle;
        match trading_style {
            TradingStyle::SteadyEarner | TradingStyle::GasHunter => 3.0,
            TradingStyle::Balanced | TradingStyle::WaveRider => 5.0,
            TradingStyle::YieldChaser => 8.0,
        }
    }

    fn calculate_opportunity_score(data: &RealtimeAPYData, _style_params: &super::ethereum::TradingStyleParams) -> f64 {
        let mut score = data.apy * 10.0; // Base score from APY

        // Deduct points for risk
        score -= data.risk_score * 5.0;

        // Bonus for preferred chains (lower gas costs)
        if data.chain != "Ethereum" {
            score += 10.0; // L2 bonus
        }

        // TVL bonus (higher TVL = more stable)
        if data.tvl_usd > 100_000_000 {
            score += 5.0;
        }

        score.max(0.0)
    }

    // Gas monitoring functions

    // Add APY opportunity to gas monitoring queue
    pub fn start_gas_monitoring_for_opportunity(
        apy_data: RealtimeAPYData,
        investment_amount_usd: f64,
        trading_style: &super::ethereum::TradingStyle
    ) -> Result<(), String> {
        let current_time = ic_cdk::api::time();

        // Calculate target gas cost based on 3-day return rule
        let daily_return_usd = (investment_amount_usd * apy_data.apy / 100.0) / 365.0;
        let target_gas_cost_usd = daily_return_usd * 3.0; // Max 3 days of returns for gas

        let opportunity = GasMonitoringOpportunity {
            apy_data: apy_data.clone(),
            investment_amount_usd,
            monitoring_started: current_time,
            target_gas_cost_usd,
            target_payback_days: 3.0,
        };

        GAS_MONITORING_QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();

            // Remove any existing monitoring for the same chain to avoid duplicates
            queue.retain(|op| op.apy_data.chain != apy_data.chain);

            // Add new opportunity
            queue.push(opportunity);
        });

        // Start gas monitoring timer if not already running
        Self::start_gas_monitoring_timer();

        ic_cdk::println!("Started gas monitoring for {} on {} with target gas cost: ${:.2}",
            apy_data.protocol, apy_data.chain, target_gas_cost_usd);

        Ok(())
    }

    // Start gas monitoring timer (runs every 5 minutes when opportunities are queued)
    fn start_gas_monitoring_timer() {
        GAS_MONITOR_TIMER.with(|timer| {
            let mut timer = timer.borrow_mut();
            if timer.is_none() {
                let timer_id = set_timer_interval(Duration::from_secs(300), || { // 5 minutes
                    ic_cdk::spawn(async {
                        Self::monitor_gas_fees_for_queued_opportunities().await;
                    });
                });
                *timer = Some(timer_id);
                ic_cdk::println!("Started gas monitoring timer (5 min intervals)");
            }
        });
    }

    // Monitor gas fees for all queued opportunities
    async fn monitor_gas_fees_for_queued_opportunities() {
        let opportunities = GAS_MONITORING_QUEUE.with(|queue| {
            queue.borrow().clone()
        });

        if opportunities.is_empty() {
            // Stop timer if no opportunities to monitor
            GAS_MONITOR_TIMER.with(|timer| {
                let mut timer = timer.borrow_mut();
                if let Some(timer_id) = timer.take() {
                    ic_cdk_timers::clear_timer(timer_id);
                    ic_cdk::println!("Stopped gas monitoring timer - no opportunities");
                }
            });
            return;
        }

        for opportunity in &opportunities {
            if let Err(e) = Self::check_gas_fee_for_opportunity(opportunity).await {
                ic_cdk::println!("Gas monitoring error for {}: {}", opportunity.apy_data.chain, e);
            }
        }

        // Remove stale opportunities (older than 1 hour)
        let current_time = ic_cdk::api::time();
        let one_hour_nanos = 60 * 60 * 1_000_000_000;

        GAS_MONITORING_QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            queue.retain(|op| current_time - op.monitoring_started < one_hour_nanos);
        });
    }

    // Check gas fee for a specific opportunity
    async fn check_gas_fee_for_opportunity(opportunity: &GasMonitoringOpportunity) -> Result<(), String> {
        let gas_data = Self::fetch_current_gas_fee(&opportunity.apy_data.chain).await?;

        // Calculate current gas cost in USD for a typical DeFi transaction
        let gas_cost_usd = Self::estimate_transaction_gas_cost_usd(&gas_data, &opportunity.apy_data.chain)?;

        ic_cdk::println!("Gas check for {} on {}: ${:.2} (target: ${:.2})",
            opportunity.apy_data.protocol,
            opportunity.apy_data.chain,
            gas_cost_usd,
            opportunity.target_gas_cost_usd
        );

        // If gas cost is acceptable, trigger the protocol switch
        if gas_cost_usd <= opportunity.target_gas_cost_usd {
            ic_cdk::println!("✅ Gas cost acceptable! Triggering protocol switch to {} on {}",
                opportunity.apy_data.protocol, opportunity.apy_data.chain);

            // Here you would trigger the actual protocol switch
            // For now, we'll just log and remove from queue
            GAS_MONITORING_QUEUE.with(|queue| {
                let mut queue = queue.borrow_mut();
                queue.retain(|op| op.apy_data.chain != opportunity.apy_data.chain);
            });
        }

        Ok(())
    }

    // Fetch current gas fees from blockchain explorers
    async fn fetch_current_gas_fee(chain: &str) -> Result<GasFeeData, String> {
        // Check cache first (cache for 1 minute)
        if let Some(cached_data) = Self::get_cached_gas_fee(chain) {
            let current_time = ic_cdk::api::time();
            let one_minute_nanos = 60 * 1_000_000_000;

            if current_time - cached_data.last_updated < one_minute_nanos {
                return Ok(cached_data);
            }
        }

        let gas_data = match chain.to_lowercase().as_str() {
            "ethereum" => Self::fetch_ethereum_gas_fee().await?,
            "arbitrum" => Self::fetch_arbitrum_gas_fee().await?,
            "optimism" => Self::fetch_optimism_gas_fee().await?,
            "polygon" => Self::fetch_polygon_gas_fee().await?,
            "base" => Self::fetch_base_gas_fee().await?,
            _ => return Err(format!("Unsupported chain for gas monitoring: {}", chain)),
        };

        // Cache the result
        Self::cache_gas_fee_data(&gas_data);

        Ok(gas_data)
    }

    // Fetch Ethereum gas fees from Etherscan
    async fn fetch_ethereum_gas_fee() -> Result<GasFeeData, String> {
        let url = "https://api.etherscan.io/api?module=gastracker&action=gasoracle";

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(2048),
            transform: Some(TransformContext::from_name("transform_gas_response".to_string(), Vec::new())),
            headers: vec![
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: "DeFlow-DeFi/1.0".to_string(),
                },
            ],
        };

        match http_request(request, 2_000_000_000).await {
            Ok((response,)) => {
                if response.status != 200u32 {
                    return Err(format!("Etherscan API error: {}", response.status));
                }

                let body_str = String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to parse response body: {}", e))?;

                let parsed: serde_json::Value = serde_json::from_str(&body_str)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

                let result = parsed["result"].as_object()
                    .ok_or("Missing result in response")?;

                // Also fetch ETH price
                let eth_price = Self::fetch_eth_price().await.unwrap_or(2000.0); // fallback price

                Ok(GasFeeData {
                    chain: "Ethereum".to_string(),
                    slow_gwei: result["SafeGasPrice"].as_str().unwrap_or("20").parse().unwrap_or(20.0),
                    standard_gwei: result["ProposeGasPrice"].as_str().unwrap_or("25").parse().unwrap_or(25.0),
                    fast_gwei: result["FastGasPrice"].as_str().unwrap_or("30").parse().unwrap_or(30.0),
                    eth_price_usd: eth_price,
                    last_updated: ic_cdk::api::time(),
                })
            }
            Err((r, m)) => Err(format!("HTTP request failed: {:?} - {}", r, m)),
        }
    }

    // Fetch ETH price for gas cost calculations
    async fn fetch_eth_price() -> Result<f64, String> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";

        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(1024),
            transform: Some(TransformContext::from_name("transform_gas_response".to_string(), Vec::new())),
            headers: vec![
                HttpHeader {
                    name: "User-Agent".to_string(),
                    value: "DeFlow-DeFi/1.0".to_string(),
                },
            ],
        };

        match http_request(request, 2_000_000_000).await {
            Ok((response,)) => {
                if response.status != 200u32 {
                    return Err(format!("CoinGecko API error: {}", response.status));
                }

                let body_str = String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to parse response body: {}", e))?;

                let parsed: serde_json::Value = serde_json::from_str(&body_str)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;

                let eth_price = parsed["ethereum"]["usd"].as_f64()
                    .ok_or("Missing ETH price in response")?;

                Ok(eth_price)
            }
            Err((r, m)) => Err(format!("HTTP request failed: {:?} - {}", r, m)),
        }
    }

    // Fetch L2 gas fees (simplified - most L2s have very low fees)
    async fn fetch_arbitrum_gas_fee() -> Result<GasFeeData, String> {
        // Arbitrum gas fees are typically very low and stable
        Ok(GasFeeData {
            chain: "Arbitrum".to_string(),
            slow_gwei: 0.1,
            standard_gwei: 0.15,
            fast_gwei: 0.2,
            eth_price_usd: Self::fetch_eth_price().await.unwrap_or(2000.0),
            last_updated: ic_cdk::api::time(),
        })
    }

    async fn fetch_optimism_gas_fee() -> Result<GasFeeData, String> {
        Ok(GasFeeData {
            chain: "Optimism".to_string(),
            slow_gwei: 0.001,
            standard_gwei: 0.001,
            fast_gwei: 0.001,
            eth_price_usd: Self::fetch_eth_price().await.unwrap_or(2000.0),
            last_updated: ic_cdk::api::time(),
        })
    }

    async fn fetch_polygon_gas_fee() -> Result<GasFeeData, String> {
        Ok(GasFeeData {
            chain: "Polygon".to_string(),
            slow_gwei: 30.0,
            standard_gwei: 35.0,
            fast_gwei: 40.0,
            eth_price_usd: 0.8, // MATIC price approximation
            last_updated: ic_cdk::api::time(),
        })
    }

    async fn fetch_base_gas_fee() -> Result<GasFeeData, String> {
        Ok(GasFeeData {
            chain: "Base".to_string(),
            slow_gwei: 0.001,
            standard_gwei: 0.001,
            fast_gwei: 0.001,
            eth_price_usd: Self::fetch_eth_price().await.unwrap_or(2000.0),
            last_updated: ic_cdk::api::time(),
        })
    }

    // Estimate transaction gas cost in USD
    fn estimate_transaction_gas_cost_usd(gas_data: &GasFeeData, chain: &str) -> Result<f64, String> {
        let gas_limit = match chain.to_lowercase().as_str() {
            "ethereum" => 150_000, // Typical DeFi transaction on Ethereum
            "arbitrum" | "optimism" | "base" => 200_000, // L2s can use more gas since it's cheaper
            "polygon" => 180_000,
            _ => 150_000,
        };

        let gas_price_gwei = gas_data.standard_gwei;
        let gas_cost_eth = (gas_limit as f64 * gas_price_gwei) / 1_000_000_000.0; // Convert gwei to ETH
        let gas_cost_usd = gas_cost_eth * gas_data.eth_price_usd;

        Ok(gas_cost_usd)
    }

    // Cache and retrieve gas fee data
    fn cache_gas_fee_data(gas_data: &GasFeeData) {
        GAS_FEE_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.insert(gas_data.chain.clone(), gas_data.clone());
        });
    }

    fn get_cached_gas_fee(chain: &str) -> Option<GasFeeData> {
        GAS_FEE_CACHE.with(|cache| {
            cache.borrow().get(chain).cloned()
        })
    }

    // Public function to check if we should trigger gas monitoring for an opportunity
    pub fn should_monitor_gas_for_opportunity(
        apy_data: &RealtimeAPYData,
        investment_amount_usd: f64,
        current_protocol: &str,
        current_chain: &str
    ) -> bool {
        // Only monitor if switching to a different protocol/chain
        if apy_data.protocol == current_protocol && apy_data.chain == current_chain {
            return false;
        }

        // Only monitor for Ethereum mainnet (L2s have predictably low fees)
        if apy_data.chain != "Ethereum" {
            return false; // L2s are cheap enough to execute immediately
        }

        // Only monitor for investments above $500 (smaller amounts can accept higher gas ratios)
        if investment_amount_usd < 500.0 {
            return false;
        }

        // Calculate potential daily return
        let daily_return_usd = (investment_amount_usd * apy_data.apy / 100.0) / 365.0;

        // Only monitor if daily return is above $1 (meaningful enough to optimize gas)
        daily_return_usd > 1.0
    }
}

// Transform functions for HTTP responses (required by IC)
#[ic_cdk::query]
fn transform_defillama_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status.clone(),
        headers: Vec::new(), // Remove sensitive headers
        body: args.response.body.clone(),
    }
}

#[ic_cdk::query]
fn transform_aave_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status.clone(),
        headers: Vec::new(),
        body: args.response.body.clone(),
    }
}

#[ic_cdk::query]
fn transform_compound_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status.clone(),
        headers: Vec::new(),
        body: args.response.body.clone(),
    }
}

#[ic_cdk::query]
fn transform_gas_response(args: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: args.response.status.clone(),
        headers: Vec::new(), // Remove sensitive headers
        body: args.response.body.clone(),
    }
}

// Public API functions
#[ic_cdk::update]
pub async fn start_apy_fetcher() -> Result<String, String> {
    let fetcher = RealtimeAPYFetcher::new();
    fetcher.start_periodic_fetch();

    // Do initial fetch
    RealtimeAPYFetcher::fetch_all_apy_data().await?;

    Ok("APY fetcher started successfully".to_string())
}

// Initialize just the timer without making HTTP calls (safe for init)
pub fn init_apy_fetcher_timer() {
    let fetcher = RealtimeAPYFetcher::new();
    fetcher.start_periodic_fetch();
    ic_cdk::println!("APY fetcher timer initialized");
}

#[ic_cdk::query]
pub fn get_current_apy_data(token: String) -> Vec<RealtimeAPYData> {
    RealtimeAPYFetcher::get_all_cached_data()
        .into_iter()
        .filter(|data| data.token.to_uppercase() == token.to_uppercase())
        .collect()
}

#[ic_cdk::query]
pub fn can_user_switch_protocol(user_id: String) -> bool {
    RealtimeAPYFetcher::can_switch_protocol(&user_id)
}

// Gas monitoring API functions
#[ic_cdk::update]
pub async fn start_gas_monitoring(
    apy_data: RealtimeAPYData,
    investment_amount_usd: f64,
    trading_style: super::ethereum::TradingStyle
) -> Result<String, String> {
    RealtimeAPYFetcher::start_gas_monitoring_for_opportunity(
        apy_data,
        investment_amount_usd,
        &trading_style
    )?;
    Ok("Gas monitoring started".to_string())
}

#[ic_cdk::query]
pub fn get_current_gas_fees(chain: String) -> Option<GasFeeData> {
    RealtimeAPYFetcher::get_cached_gas_fee(&chain)
}

#[ic_cdk::query]
pub fn should_monitor_gas(
    apy_data: RealtimeAPYData,
    investment_amount_usd: f64,
    current_protocol: String,
    current_chain: String
) -> bool {
    RealtimeAPYFetcher::should_monitor_gas_for_opportunity(
        &apy_data,
        investment_amount_usd,
        &current_protocol,
        &current_chain
    )
}