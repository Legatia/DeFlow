// Cross-Chain Bridge Router
// Intelligent routing for optimal yield farming across chains
// Selects best bridge based on cost, speed, and reliability

use super::*;
use super::wormhole::WormholeService;
use super::layerzero::LayerZeroService;
use super::stargate::StargateService;
use std::collections::HashMap;

/// Bridge router for multi-protocol optimization
#[derive(Debug, Clone)]
pub struct BridgeRouter {
    pub wormhole: WormholeService,
    pub layerzero: LayerZeroService,
    pub stargate: StargateService,
    pub route_cache: HashMap<String, CachedRoute>,
    pub cache_ttl: u64, // seconds
}

#[derive(Debug, Clone)]
struct CachedRoute {
    route: BridgeRoute,
    cached_at: u64,
}

impl BridgeRouter {
    pub fn new(wormhole_network: super::wormhole::WormholeNetwork) -> Self {
        Self {
            wormhole: WormholeService::new(wormhole_network),
            layerzero: LayerZeroService::new(),
            stargate: StargateService::new(),
            route_cache: HashMap::new(),
            cache_ttl: 300, // 5 minutes
        }
    }

    /// Get optimal bridge route for yield farming
    pub async fn get_optimal_route(
        &mut self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
        priority: RoutingPriority,
    ) -> Result<BridgeRoute, BridgeError> {
        // Check cache first
        let cache_key = format!("{:?}_{:?}_{}", from_chain, to_chain, asset);
        if let Some(cached) = self.route_cache.get(&cache_key) {
            if ic_cdk::api::time() - cached.cached_at < self.cache_ttl * 1_000_000_000 {
                return Ok(cached.route.clone());
            }
        }

        // Get all available routes
        let mut routes = self.get_all_routes(from_chain.clone(), to_chain.clone(), asset.clone(), amount).await?;

        if routes.is_empty() {
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        // Select best route based on priority
        let best_route = match priority {
            RoutingPriority::LowestCost => {
                routes.sort_by(|a, b| a.estimated_fee.partial_cmp(&b.estimated_fee).unwrap());
                routes.into_iter().next().unwrap()
            }
            RoutingPriority::Fastest => {
                routes.sort_by(|a, b| a.estimated_time_seconds.cmp(&b.estimated_time_seconds));
                routes.into_iter().next().unwrap()
            }
            RoutingPriority::MostSecure => {
                routes.sort_by(|a, b| b.security_score.partial_cmp(&a.security_score).unwrap());
                routes.into_iter().next().unwrap()
            }
            RoutingPriority::Balanced => {
                // Weighted score: 40% cost, 30% speed, 30% security
                routes.sort_by(|a, b| {
                    let score_a = self.calculate_balanced_score(a);
                    let score_b = self.calculate_balanced_score(b);
                    score_b.partial_cmp(&score_a).unwrap()
                });
                routes.into_iter().next().unwrap()
            }
        };

        // Cache the result
        self.route_cache.insert(cache_key, CachedRoute {
            route: best_route.clone(),
            cached_at: ic_cdk::api::time(),
        });

        Ok(best_route)
    }

    /// Get all available bridge routes
    async fn get_all_routes(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<Vec<BridgeRoute>, BridgeError> {
        let mut routes = Vec::new();

        // Try Stargate (best for stablecoins)
        if self.stargate.is_route_supported(&from_chain, &to_chain, &asset) {
            if let Ok(route) = self.stargate.get_bridge_route(
                from_chain.clone(),
                to_chain.clone(),
                asset.clone(),
                amount,
            ).await {
                routes.push(route);
            }
        }

        // Try LayerZero (fastest, good security)
        if self.layerzero.is_route_supported(&from_chain, &to_chain) {
            if let Ok(route) = self.layerzero.get_bridge_route(
                from_chain.clone(),
                to_chain.clone(),
                asset.clone(),
                amount,
            ).await {
                routes.push(route);
            }
        }

        // Try Wormhole (most chain coverage)
        if self.wormhole.is_route_supported(&from_chain, &to_chain) {
            if let Ok(route) = self.wormhole.get_bridge_route(
                from_chain,
                to_chain,
                asset,
                amount,
            ).await {
                routes.push(route);
            }
        }

        Ok(routes)
    }

    /// Calculate balanced score for route
    fn calculate_balanced_score(&self, route: &BridgeRoute) -> f64 {
        // Normalize cost (lower is better)
        let cost_score = 1.0 / (1.0 + route.estimated_fee / 100.0);

        // Normalize time (lower is better)
        let time_score = 1.0 / (1.0 + route.estimated_time_seconds as f64 / 600.0);

        // Security score (higher is better)
        let security_score = route.security_score;

        // Weighted average
        (cost_score * 0.4) + (time_score * 0.3) + (security_score * 0.3)
    }

    /// Execute bridge with automatic fallback
    pub async fn execute_bridge(
        &self,
        route: BridgeRoute,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
        recipient: String,
    ) -> Result<BridgeTransaction, BridgeError> {
        match route.protocol {
            BridgeProtocol::Stargate => {
                self.stargate.bridge_transfer(
                    from_chain,
                    to_chain,
                    asset,
                    amount,
                    recipient,
                    None, // No slippage protection for now
                ).await
            }
            BridgeProtocol::LayerZero => {
                self.layerzero.bridge_transfer(
                    from_chain,
                    to_chain,
                    asset,
                    amount,
                    recipient,
                ).await
            }
            BridgeProtocol::Wormhole => {
                self.wormhole.bridge_transfer(
                    from_chain,
                    to_chain,
                    asset,
                    amount,
                    recipient,
                ).await
            }
            _ => Err(BridgeError::UnsupportedRoute(from_chain, to_chain)),
        }
    }

    /// Get best route for specific use case
    pub async fn get_route_for_yield_farming(
        &mut self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<BridgeRoute, BridgeError> {
        // For yield farming, prioritize: low cost > security > speed
        self.get_optimal_route(from_chain, to_chain, asset, amount, RoutingPriority::LowestCost).await
    }

    /// Get best route for arbitrage
    pub async fn get_route_for_arbitrage(
        &mut self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<BridgeRoute, BridgeError> {
        // For arbitrage, prioritize: speed > low cost > security
        self.get_optimal_route(from_chain, to_chain, asset, amount, RoutingPriority::Fastest).await
    }

    /// Compare routes side-by-side
    pub async fn compare_routes(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<RouteComparison, BridgeError> {
        let routes = self.get_all_routes(from_chain.clone(), to_chain.clone(), asset.clone(), amount).await?;

        if routes.is_empty() {
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        let cheapest = routes.iter().min_by(|a, b| a.estimated_fee.partial_cmp(&b.estimated_fee).unwrap()).cloned();
        let fastest = routes.iter().min_by(|a, b| a.estimated_time_seconds.cmp(&b.estimated_time_seconds)).cloned();
        let most_secure = routes.iter().max_by(|a, b| a.security_score.partial_cmp(&b.security_score).unwrap()).cloned();

        Ok(RouteComparison {
            all_routes: routes,
            cheapest,
            fastest,
            most_secure,
        })
    }

    /// Clear expired cache entries
    pub fn clear_cache(&mut self) {
        let now = ic_cdk::api::time();
        let ttl_nanos = self.cache_ttl * 1_000_000_000;

        self.route_cache.retain(|_, cached| {
            now - cached.cached_at < ttl_nanos
        });
    }
}

/// Routing priority for bridge selection
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RoutingPriority {
    LowestCost,   // Minimize fees
    Fastest,      // Minimize time
    MostSecure,   // Maximize security
    Balanced,     // Balance all factors
}

/// Route comparison result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RouteComparison {
    pub all_routes: Vec<BridgeRoute>,
    pub cheapest: Option<BridgeRoute>,
    pub fastest: Option<BridgeRoute>,
    pub most_secure: Option<BridgeRoute>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = BridgeRouter::new(super::wormhole::WormholeNetwork::Testnet);
        assert_eq!(router.cache_ttl, 300);
    }

    #[test]
    fn test_balanced_score() {
        let router = BridgeRouter::new(super::wormhole::WormholeNetwork::Testnet);

        let route = BridgeRoute {
            protocol: BridgeProtocol::Stargate,
            from_chain: ChainId::Polygon,
            to_chain: ChainId::Arbitrum,
            asset: "USDC".to_string(),
            estimated_fee: 5.0,
            estimated_time_seconds: 300,
            min_amount: 1_000_000,
            max_amount: 1_000_000_000,
            security_score: 0.9,
            success_rate_24h: 0.99,
            available: true,
        };

        let score = router.calculate_balanced_score(&route);
        assert!(score > 0.0 && score <= 1.0);
    }
}
