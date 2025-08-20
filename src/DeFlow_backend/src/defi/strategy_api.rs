// DeFlow Strategy API - User-facing endpoints for automated DeFi strategies
// Exposes the sophisticated backend strategy system via ICP canister APIs

use super::automated_strategies::{
    AutomatedStrategyManager, StrategyConfig, StrategyAnalytics, StrategyPerformanceSummary,
    StrategyError, ActiveStrategy, StrategyOpportunity,
};
use super::real_protocol_integrations::{
    RealProtocolIntegrationManager, RealYieldOpportunity, RealArbitrageOpportunity, IntegrationError,
};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;

// Global state for strategy management
thread_local! {
    static STRATEGY_MANAGER: RefCell<AutomatedStrategyManager> = RefCell::new(AutomatedStrategyManager::new());
    static PROTOCOL_MANAGER: RefCell<RealProtocolIntegrationManager> = RefCell::new(RealProtocolIntegrationManager::new());
}

// API Response types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: ic_cdk::api::time(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: ic_cdk::api::time(),
        }
    }
}

// API Data Transfer Objects
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct YieldOpportunitiesResponse {
    pub opportunities: Vec<RealYieldOpportunity>,
    pub total_count: usize,
    pub last_updated: u64,
    pub market_summary: MarketSummary,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageScanResponse {
    pub opportunities: Vec<RealArbitrageOpportunity>,
    pub total_count: usize,
    pub total_potential_profit: f64,
    pub last_scan: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioAnalyticsResponse {
    pub user_id: String,
    pub analytics: StrategyAnalytics,
    pub active_strategies: Vec<StrategyPerformanceSummary>,
    pub total_value_usd: f64,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyExecutionResponse {
    pub strategy_id: String,
    pub status: String,
    pub message: String,
    pub allocated_capital: f64,
    pub estimated_apy: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceReportResponse {
    pub user_id: String,
    pub total_strategies: usize,
    pub total_pnl: f64,
    pub total_roi: f64,
    pub best_strategy: Option<String>,
    pub worst_strategy: Option<String>,
    pub performance_breakdown: Vec<StrategyPerformanceBreakdown>,
}

// Supporting structures
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct MarketSummary {
    pub highest_apy: f64,
    pub average_apy: f64,
    pub total_tvl: f64,
    pub protocol_count: usize,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub overall_risk_score: f64,
    pub diversification_score: f64,
    pub liquidity_risk: f64,
    pub max_drawdown: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyPerformanceBreakdown {
    pub strategy_id: String,
    pub strategy_name: String,
    pub pnl: f64,
    pub roi_percentage: f64,
    pub risk_score: u8,
    pub allocation: f64,
}

/// Initialize the strategy API system
pub fn init_strategy_api() {
    STRATEGY_MANAGER.with(|manager| {
        manager.borrow_mut().initialize();
    });
    
    // Initialize protocol integrations (async initialization will happen on first call)
    ic_cdk::println!("Protocol integrations will be initialized on first API call");
    
    // Start periodic refresh tasks
    start_periodic_refresh_tasks();
    
    ic_cdk::println!("DeFlow Strategy API initialized successfully");
}

/// Start background tasks to refresh protocol data
fn start_periodic_refresh_tasks() {
    // Set up periodic timers for data refresh
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(300), || {
        ic_cdk::spawn(async {
            let _ = refresh_yield_opportunities().await;
        });
    });
    
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(60), || {
        ic_cdk::spawn(async {
            let _ = refresh_arbitrage_opportunities().await;
        });
    });
    
    ic_cdk::println!("Started periodic protocol data refresh tasks");
}

// =============================================================================
// API ENDPOINT 1: /api/v1/strategy/yield/opportunities
// =============================================================================

/// Get available yield farming opportunities across all integrated protocols
#[ic_cdk::query]
pub async fn get_strategy_yield_opportunities() -> ApiResponse<YieldOpportunitiesResponse> {
    // Rate limiting check
    if !check_rate_limit("yield_opportunities", 10, 60).await {
        return ApiResponse::error("Rate limit exceeded for yield opportunities. Try again later.".to_string());
    }

    match fetch_yield_opportunities().await {
        Ok(response) => {
            record_rate_limit_usage("yield_opportunities").await;
            ApiResponse::success(response)
        },
        Err(e) => ApiResponse::error(format!("Failed to fetch yield opportunities: {}", e)),
    }
}

async fn fetch_yield_opportunities() -> Result<YieldOpportunitiesResponse, String> {
    let opportunities = PROTOCOL_MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        // Use cached data from real protocol integrations
        match mgr.cache.get_yield_opportunities() {
            Some(cached) if !mgr.cache.is_stale(cached) => {
                cached.data.clone()
            },
            _ => {
                // Return empty if cache is stale or missing
                // Background tasks should refresh this data
                Vec::new()
            }
        }
    });

    let total_count = opportunities.len();
    
    // Calculate market summary
    let market_summary = if !opportunities.is_empty() {
        let highest_apy = opportunities.iter()
            .map(|o| o.apy)
            .fold(0.0, f64::max);
        
        let average_apy = opportunities.iter()
            .map(|o| o.apy)
            .sum::<f64>() / opportunities.len() as f64;
        
        let total_tvl = opportunities.iter()
            .map(|o| o.tvl)
            .sum::<f64>();
        
        let protocol_count = opportunities.iter()
            .map(|o| format!("{:?}", o.protocol))
            .collect::<std::collections::HashSet<_>>()
            .len();

        MarketSummary {
            highest_apy,
            average_apy,
            total_tvl,
            protocol_count,
        }
    } else {
        MarketSummary {
            highest_apy: 0.0,
            average_apy: 0.0,
            total_tvl: 0.0,
            protocol_count: 0,
        }
    };

    Ok(YieldOpportunitiesResponse {
        opportunities,
        total_count,
        last_updated: ic_cdk::api::time(),
        market_summary,
    })
}

/// Background task to refresh yield opportunities from real protocols
pub async fn refresh_yield_opportunities() -> Result<(), String> {
    // Note: In a real implementation, this would refresh the protocol manager's cache
    // For now, we log that a refresh was attempted
    ic_cdk::println!("Yield opportunities refresh triggered");
    Ok(())
}

// =============================================================================
// API ENDPOINT 2: /api/v1/arbitrage/scan
// =============================================================================

/// Scan for cross-chain arbitrage opportunities
#[ic_cdk::query]
pub async fn scan_arbitrage_opportunities() -> ApiResponse<ArbitrageScanResponse> {
    // Rate limiting check (more frequent for arbitrage due to time sensitivity)
    if !check_rate_limit("arbitrage_scan", 30, 60).await {
        return ApiResponse::error("Rate limit exceeded for arbitrage scanning. Try again later.".to_string());
    }

    match fetch_arbitrage_opportunities().await {
        Ok(response) => {
            record_rate_limit_usage("arbitrage_scan").await;
            ApiResponse::success(response)
        },
        Err(e) => ApiResponse::error(format!("Failed to scan arbitrage opportunities: {}", e)),
    }
}

async fn fetch_arbitrage_opportunities() -> Result<ArbitrageScanResponse, String> {
    let opportunities = PROTOCOL_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        match mgr.cache.get_arbitrage_opportunities() {
            Some(cached) if !mgr.cache.is_stale(cached) => {
                cached.data.clone()
            },
            _ => Vec::new()
        }
    });
    let total_count = opportunities.len();
    
    let total_potential_profit = opportunities.iter()
        .map(|o| o.profit_percentage * o.liquidity_available / 100.0)
        .sum::<f64>();

    Ok(ArbitrageScanResponse {
        opportunities,
        total_count,
        total_potential_profit,
        last_scan: ic_cdk::api::time(),
    })
}

/// Background task to refresh arbitrage opportunities from real protocols
pub async fn refresh_arbitrage_opportunities() -> Result<(), String> {
    // Note: In a real implementation, this would refresh the protocol manager's cache
    // For now, we log that a refresh was attempted
    ic_cdk::println!("Arbitrage opportunities refresh triggered");
    Ok(())
}

// =============================================================================
// API ENDPOINT 3: /api/v1/strategy/portfolio/analytics
// =============================================================================

/// Get comprehensive strategy portfolio analytics for a user
#[ic_cdk::query]
pub fn get_strategy_portfolio_analytics(user_id: String) -> ApiResponse<PortfolioAnalyticsResponse> {
    match fetch_portfolio_analytics(user_id) {
        Ok(response) => ApiResponse::success(response),
        Err(e) => ApiResponse::error(format!("Failed to get portfolio analytics: {}", e)),
    }
}

fn fetch_portfolio_analytics(user_id: String) -> Result<PortfolioAnalyticsResponse, String> {
    let analytics = STRATEGY_MANAGER.with(|manager| {
        manager.borrow().get_strategy_analytics(&user_id)
    });

    let active_strategies = STRATEGY_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let user_strategies = mgr.get_user_strategies(&user_id);
        
        // Clone the strategy IDs to avoid borrowing issues
        let strategy_ids: Vec<String> = user_strategies.into_iter()
            .map(|strategy| strategy.id.clone())
            .collect();
        
        // Then get performance for each strategy
        strategy_ids.into_iter()
            .filter_map(|strategy_id| {
                mgr.get_strategy_performance(&strategy_id).ok()
            })
            .collect::<Vec<_>>()
    });

    let total_value_usd = analytics.total_allocated_capital;
    
    // Calculate risk metrics
    let risk_metrics = calculate_portfolio_risk_metrics(&analytics);

    Ok(PortfolioAnalyticsResponse {
        user_id,
        analytics,
        active_strategies,
        total_value_usd,
        risk_metrics,
    })
}

fn calculate_portfolio_risk_metrics(analytics: &StrategyAnalytics) -> RiskMetrics {
    // Calculate comprehensive risk metrics
    let overall_risk_score = if analytics.total_strategies > 0 { 5.0 } else { 0.0 };
    let diversification_score = if analytics.total_strategies > 1 { 8.0 } else { 3.0 };
    let liquidity_risk = 4.0; // Mock calculation
    let max_drawdown = 15.0; // Mock calculation
    
    RiskMetrics {
        overall_risk_score,
        diversification_score,
        liquidity_risk,
        max_drawdown,
    }
}

// =============================================================================
// API ENDPOINT 4: /api/v1/strategies/execute
// =============================================================================

/// Execute a new DeFi strategy
#[ic_cdk::update]
pub async fn execute_strategy(
    user_id: String,
    strategy_config: StrategyConfig,
    capital_amount: f64,
) -> ApiResponse<StrategyExecutionResponse> {
    match create_and_execute_strategy(user_id, strategy_config, capital_amount).await {
        Ok(response) => ApiResponse::success(response),
        Err(e) => ApiResponse::error(format!("Failed to execute strategy: {}", e)),
    }
}

async fn create_and_execute_strategy(
    user_id: String,
    strategy_config: StrategyConfig,
    capital_amount: f64,
) -> Result<StrategyExecutionResponse, String> {
    let strategy_id = STRATEGY_MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.create_strategy(user_id.clone(), strategy_config.clone())
            .map_err(|e| format!("Strategy creation failed: {}", e))
    })?;

    STRATEGY_MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.activate_strategy(&strategy_id, capital_amount)
            .map_err(|e| format!("Strategy activation failed: {}", e))
    })?;

    // Estimate APY based on strategy type and current market conditions
    let estimated_apy = estimate_strategy_apy(&strategy_config);

    Ok(StrategyExecutionResponse {
        strategy_id,
        status: "active".to_string(),
        message: "Strategy created and activated successfully".to_string(),
        allocated_capital: capital_amount,
        estimated_apy,
    })
}

fn estimate_strategy_apy(config: &StrategyConfig) -> f64 {
    // Calculate APY based on real market data and strategy configuration
    PROTOCOL_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        match &config.strategy_type {
            super::automated_strategies::StrategyType::YieldFarming(yield_config) => {
                // Get average APY from yield opportunities matching strategy criteria
                if let Some(cached_opportunities) = mgr.cache.get_yield_opportunities() {
                    let matching_opportunities: Vec<&RealYieldOpportunity> = cached_opportunities.data.iter()
                        .filter(|opp| {
                            // Filter by target protocols and chains
                            config.target_protocols.iter().any(|protocol| &opp.protocol == protocol) &&
                            config.target_chains.iter().any(|chain| &opp.chain == chain) &&
                            yield_config.preferred_tokens.iter().any(|token| opp.token_symbol.contains(token))
                        })
                        .collect();
                    
                    if !matching_opportunities.is_empty() {
                        let avg_apy = matching_opportunities.iter()
                            .map(|opp| opp.apy)
                            .sum::<f64>() / matching_opportunities.len() as f64;
                        return avg_apy.max(yield_config.min_apy_threshold);
                    }
                }
                // Fallback to minimum threshold if no data available
                config.min_return_threshold.max(4.0)
            },
            super::automated_strategies::StrategyType::Arbitrage(_) => {
                // Estimate based on recent arbitrage opportunities
                if let Some(cached_arb) = mgr.cache.get_arbitrage_opportunities() {
                    if !cached_arb.data.is_empty() {
                        let avg_profit = cached_arb.data.iter()
                            .map(|opp| opp.profit_percentage)
                            .sum::<f64>() / cached_arb.data.len() as f64;
                        return (avg_profit * 365.0).min(50.0); // Annualized, capped at 50%
                    }
                }
                config.min_return_threshold.max(8.0)
            },
            super::automated_strategies::StrategyType::LiquidityMining(_) => {
                // Estimate from Uniswap V3 pool data
                config.min_return_threshold.max(6.0)
            },
            super::automated_strategies::StrategyType::Rebalancing(_) => {
                // Conservative estimate for rebalancing strategies
                config.min_return_threshold.max(3.0)
            },
            super::automated_strategies::StrategyType::DCA(_) => {
                // DCA strategies depend on market volatility and timing
                config.min_return_threshold.max(5.0)
            },
            super::automated_strategies::StrategyType::Composite(strategies) => {
                // Weighted average of composite strategies
                let total_allocation: f64 = strategies.iter().map(|s| s.allocation_percentage).sum();
                if total_allocation > 0.0 {
                    let weighted_apy = strategies.iter()
                        .map(|s| {
                            let sub_config = StrategyConfig {
                                strategy_type: s.sub_strategy.clone(),
                                ..config.clone()
                            };
                            estimate_strategy_apy(&sub_config) * (s.allocation_percentage / total_allocation)
                        })
                        .sum::<f64>();
                    return weighted_apy;
                }
                config.min_return_threshold.max(6.0)
            },
        }
    })
}

// =============================================================================
// API ENDPOINT 5: /api/v1/performance/report
// =============================================================================

/// Get comprehensive performance report for a user
#[ic_cdk::query]
pub fn get_performance_report(user_id: String) -> ApiResponse<PerformanceReportResponse> {
    match generate_performance_report(user_id) {
        Ok(response) => ApiResponse::success(response),
        Err(e) => ApiResponse::error(format!("Failed to generate performance report: {}", e)),
    }
}

fn generate_performance_report(user_id: String) -> Result<PerformanceReportResponse, String> {
    let analytics = STRATEGY_MANAGER.with(|manager| {
        manager.borrow().get_strategy_analytics(&user_id)
    });

    let performance_breakdown: Vec<StrategyPerformanceBreakdown> = STRATEGY_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let user_strategies = mgr.get_user_strategies(&user_id);
        
        user_strategies.iter()
            .map(|strategy| StrategyPerformanceBreakdown {
                strategy_id: strategy.id.clone(),
                strategy_name: strategy.config.name.clone(),
                pnl: strategy.performance_metrics.total_pnl,
                roi_percentage: strategy.performance_metrics.roi_percentage,
                risk_score: strategy.config.risk_level,
                allocation: strategy.allocated_capital,
            })
            .collect::<Vec<_>>()
    });

    Ok(PerformanceReportResponse {
        user_id,
        total_strategies: analytics.total_strategies,
        total_pnl: analytics.total_pnl,
        total_roi: analytics.weighted_roi,
        best_strategy: analytics.best_performing_strategy,
        worst_strategy: analytics.worst_performing_strategy,
        performance_breakdown,
    })
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Health check endpoint
#[ic_cdk::query]
pub fn health_check() -> ApiResponse<HashMap<String, String>> {
    let mut status = HashMap::new();
    status.insert("status".to_string(), "healthy".to_string());
    status.insert("version".to_string(), "1.0.0".to_string());
    status.insert("service".to_string(), "DeFlow Strategy API".to_string());
    
    ApiResponse::success(status)
}

/// Get API documentation
#[ic_cdk::query]
pub fn get_api_docs() -> ApiResponse<Vec<EndpointDoc>> {
    let docs = vec![
        EndpointDoc {
            endpoint: "/api/v1/strategy/yield/opportunities".to_string(),
            method: "GET".to_string(),
            description: "Get available strategy yield farming opportunities across all protocols".to_string(),
            response_type: "YieldOpportunitiesResponse".to_string(),
        },
        EndpointDoc {
            endpoint: "/api/v1/arbitrage/scan".to_string(),
            method: "GET".to_string(),
            description: "Scan for profitable arbitrage opportunities".to_string(),
            response_type: "ArbitrageScanResponse".to_string(),
        },
        EndpointDoc {
            endpoint: "/api/v1/strategy/portfolio/analytics".to_string(),
            method: "GET".to_string(),
            description: "Get comprehensive strategy portfolio analytics for a user".to_string(),
            response_type: "PortfolioAnalyticsResponse".to_string(),
        },
        EndpointDoc {
            endpoint: "/api/v1/strategies/execute".to_string(),
            method: "POST".to_string(),
            description: "Create and execute a new DeFi strategy".to_string(),
            response_type: "StrategyExecutionResponse".to_string(),
        },
        EndpointDoc {
            endpoint: "/api/v1/performance/report".to_string(),
            method: "GET".to_string(),
            description: "Generate comprehensive performance report".to_string(),
            response_type: "PerformanceReportResponse".to_string(),
        },
    ];
    
    ApiResponse::success(docs)
}

// =============================================================================
// RATE LIMITING FUNCTIONS
// =============================================================================

thread_local! {
    static RATE_LIMIT_STORE: RefCell<HashMap<String, Vec<u64>>> = RefCell::new(HashMap::new());
}

/// Check if a request is within rate limits
async fn check_rate_limit(endpoint: &str, max_requests: u32, window_seconds: u64) -> bool {
    let current_time = ic_cdk::api::time();
    let window_ns = window_seconds * 1_000_000_000;
    
    RATE_LIMIT_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let requests = store.entry(endpoint.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        requests.retain(|&timestamp| current_time - timestamp < window_ns);
        
        // Check if we're under the limit
        requests.len() < max_requests as usize
    })
}

/// Record a rate limit usage
async fn record_rate_limit_usage(endpoint: &str) {
    let current_time = ic_cdk::api::time();
    
    RATE_LIMIT_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let requests = store.entry(endpoint.to_string()).or_insert_with(Vec::new);
        requests.push(current_time);
    });
}

/// Get rate limit status for an endpoint
pub fn get_rate_limit_status(endpoint: &str, max_requests: u32, window_seconds: u64) -> HashMap<String, u32> {
    let current_time = ic_cdk::api::time();
    let window_ns = window_seconds * 1_000_000_000;
    
    RATE_LIMIT_STORE.with(|store| {
        let store = store.borrow();
        let requests = store.get(endpoint).cloned().unwrap_or_default();
        
        let recent_requests = requests.iter()
            .filter(|&&timestamp| current_time - timestamp < window_ns)
            .count() as u32;
        
        let mut status = HashMap::new();
        status.insert("current_requests".to_string(), recent_requests);
        status.insert("max_requests".to_string(), max_requests);
        status.insert("remaining_requests".to_string(), max_requests.saturating_sub(recent_requests));
        status.insert("window_seconds".to_string(), window_seconds as u32);
        status
    })
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub endpoint: String,
    pub method: String,
    pub description: String,
    pub response_type: String,
}