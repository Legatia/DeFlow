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
    
    ic_cdk::println!("DeFlow Strategy API initialized successfully");
}

// =============================================================================
// API ENDPOINT 1: /api/v1/strategy/yield/opportunities
// =============================================================================

/// Get available yield farming opportunities across all integrated protocols
#[ic_cdk::query]
pub async fn get_strategy_yield_opportunities() -> ApiResponse<YieldOpportunitiesResponse> {
    match fetch_yield_opportunities().await {
        Ok(response) => ApiResponse::success(response),
        Err(e) => ApiResponse::error(format!("Failed to fetch yield opportunities: {}", e)),
    }
}

async fn fetch_yield_opportunities() -> Result<YieldOpportunitiesResponse, String> {
    let opportunities = PROTOCOL_MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        // Use a mock async operation since we can't actually make async calls in query
        // In a real implementation, this would be cached data updated by periodic tasks
        get_cached_yield_opportunities(&mut mgr)
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

// Mock function for cached opportunities - in production this would be real cached data
fn get_cached_yield_opportunities(_manager: &mut RealProtocolIntegrationManager) -> Vec<RealYieldOpportunity> {
    // DEMO: Enhanced mock opportunities with realistic current market data
    vec![
        RealYieldOpportunity {
            protocol: crate::defi::yield_farming::DeFiProtocol::Aave,
            chain: crate::defi::yield_farming::ChainId::Ethereum,
            token_symbol: "USDC".to_string(),
            apy: 3.8, // More realistic current yield
            tvl: 750_000_000.0, // Updated TVL
            risk_score: 2, // Aave is lower risk
            min_deposit: 100.0,
            max_deposit: 100_000.0,
            liquidity_available: 50_000_000.0,
            last_updated: ic_cdk::api::time(),
        },
        RealYieldOpportunity {
            protocol: crate::defi::yield_farming::DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3),
            chain: crate::defi::yield_farming::ChainId::Ethereum,
            token_symbol: "ETH/USDC".to_string(),
            apy: 8.5,
            tvl: 200_000_000.0,
            risk_score: 5,
            min_deposit: 500.0,
            max_deposit: 50_000.0,
            liquidity_available: 20_000_000.0,
            last_updated: ic_cdk::api::time(),
        },
    ]
}

// =============================================================================
// API ENDPOINT 2: /api/v1/arbitrage/scan
// =============================================================================

/// Scan for cross-chain arbitrage opportunities
#[ic_cdk::query]
pub async fn scan_arbitrage_opportunities() -> ApiResponse<ArbitrageScanResponse> {
    match fetch_arbitrage_opportunities().await {
        Ok(response) => ApiResponse::success(response),
        Err(e) => ApiResponse::error(format!("Failed to scan arbitrage opportunities: {}", e)),
    }
}

async fn fetch_arbitrage_opportunities() -> Result<ArbitrageScanResponse, String> {
    let opportunities = get_cached_arbitrage_opportunities();
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

fn get_cached_arbitrage_opportunities() -> Vec<RealArbitrageOpportunity> {
    // Return mock arbitrage opportunities - in production this would be real cached data
    vec![
        RealArbitrageOpportunity {
            token_symbol: "ETH".to_string(),
            buy_dex: "Uniswap".to_string(),
            sell_dex: "Curve".to_string(),
            buy_price: 2000.0,
            sell_price: 2015.0,
            profit_percentage: 0.75,
            estimated_gas_cost: 50.0,
            liquidity_available: 100_000.0,
            expires_at: ic_cdk::api::time() + (5 * 60 * 1_000_000_000),
            discovered_at: ic_cdk::api::time(),
        },
    ]
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
    // Mock APY estimation based on strategy type
    match &config.strategy_type {
        super::automated_strategies::StrategyType::YieldFarming(_) => 6.5,
        super::automated_strategies::StrategyType::Arbitrage(_) => 12.0,
        super::automated_strategies::StrategyType::LiquidityMining(_) => 8.0,
        super::automated_strategies::StrategyType::Rebalancing(_) => 4.5,
        super::automated_strategies::StrategyType::DCA(_) => 7.0,
        super::automated_strategies::StrategyType::Composite(_) => 9.0,
    }
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

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub endpoint: String,
    pub method: String,
    pub description: String,
    pub response_type: String,
}