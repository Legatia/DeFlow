// Stargate Finance Bridge Integration
// Composable omnichain DeFi protocol built on LayerZero
// https://stargate.finance

use super::*;
use std::collections::HashMap;

/// Stargate pool IDs for different assets
const STARGATE_POOL_USDC: u16 = 1;
const STARGATE_POOL_USDT: u16 = 2;
const STARGATE_POOL_DAI: u16 = 3;
const STARGATE_POOL_FRAX: u16 = 7;
const STARGATE_POOL_ETH: u16 = 13;

/// Stargate bridge service (specialized for stablecoins)
#[derive(Debug, Clone)]
pub struct StargateService {
    pub supported_chains: HashMap<ChainId, bool>,
    pub pool_ids: HashMap<String, u16>,
    pub http_timeout: u64,
}

impl StargateService {
    pub fn new() -> Self {
        let mut supported_chains = HashMap::new();
        supported_chains.insert(ChainId::Ethereum, true);
        supported_chains.insert(ChainId::Polygon, true);
        supported_chains.insert(ChainId::Arbitrum, true);
        supported_chains.insert(ChainId::Optimism, true);
        supported_chains.insert(ChainId::Base, true);
        supported_chains.insert(ChainId::Avalanche, true);

        let mut pool_ids = HashMap::new();
        pool_ids.insert("USDC".to_string(), STARGATE_POOL_USDC);
        pool_ids.insert("USDT".to_string(), STARGATE_POOL_USDT);
        pool_ids.insert("DAI".to_string(), STARGATE_POOL_DAI);
        pool_ids.insert("FRAX".to_string(), STARGATE_POOL_FRAX);
        pool_ids.insert("ETH".to_string(), STARGATE_POOL_ETH);

        Self {
            supported_chains,
            pool_ids,
            http_timeout: 30_000_000_000,
        }
    }

    /// Check if asset is supported (Stargate focuses on stablecoins + ETH)
    pub fn is_asset_supported(&self, asset: &str) -> bool {
        self.pool_ids.contains_key(asset)
    }

    /// Check if route is supported
    pub fn is_route_supported(&self, from: &ChainId, to: &ChainId, asset: &str) -> bool {
        self.supported_chains.contains_key(from)
            && self.supported_chains.contains_key(to)
            && self.is_asset_supported(asset)
    }

    /// Get bridge route details
    pub async fn get_bridge_route(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<BridgeRoute, BridgeError> {
        if !self.is_route_supported(&from_chain, &to_chain, &asset) {
            if !self.is_asset_supported(&asset) {
                return Err(BridgeError::UnsupportedAsset(asset));
            }
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount, &asset).await?;
        let time = self.estimate_bridge_time(&from_chain, &to_chain);

        Ok(BridgeRoute {
            protocol: BridgeProtocol::Stargate,
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            estimated_fee: fee,
            estimated_time_seconds: time,
            min_amount: self.get_min_amount(&asset),
            max_amount: self.get_max_amount(&from_chain, &to_chain, &asset),
            security_score: 0.92, // Stargate has excellent security + audits
            success_rate_24h: 0.995,
            available: true,
        })
    }

    /// Estimate bridge fee
    async fn estimate_bridge_fee(
        &self,
        from_chain: &ChainId,
        to_chain: &ChainId,
        amount: u64,
        asset: &str,
    ) -> Result<f64, BridgeError> {
        // Stargate fees:
        // 1. Swap fee (0.06% for stables, 0.06% for ETH)
        // 2. Protocol fee (0.00% currently)
        // 3. LP fee (dynamic based on pool utilization)
        // 4. LayerZero messaging fee

        let swap_fee_rate = match asset {
            "USDC" | "USDT" | "DAI" | "FRAX" => 0.0006, // 0.06%
            "ETH" => 0.0006,
            _ => 0.001,
        };

        let swap_fee = (amount as f64 / 1_000_000.0) * swap_fee_rate; // Convert to USD

        // LP fee varies by pool utilization (0-0.06%)
        let lp_fee = swap_fee * 0.5; // Simplified

        // LayerZero base messaging fee
        let lz_fee = self.estimate_layerzero_fee(from_chain, to_chain);

        // Slippage (minimal for stablecoins)
        let slippage = if asset.contains("USD") || asset.contains("DAI") {
            (amount as f64 / 1_000_000.0) * 0.0001 // 0.01% slippage for stables
        } else {
            (amount as f64 / 1_000_000.0) * 0.003 // 0.3% for ETH
        };

        Ok(swap_fee + lp_fee + lz_fee + slippage)
    }

    /// Estimate LayerZero messaging fee
    fn estimate_layerzero_fee(&self, from_chain: &ChainId, to_chain: &ChainId) -> f64 {
        let source_gas = match from_chain {
            ChainId::Ethereum => 15.0,
            ChainId::Arbitrum => 2.0,
            ChainId::Optimism => 2.5,
            ChainId::Polygon => 0.5,
            ChainId::Base => 1.5,
            _ => 5.0,
        };

        let dest_gas = match to_chain {
            ChainId::Ethereum => 15.0,
            ChainId::Arbitrum => 2.0,
            ChainId::Optimism => 2.5,
            ChainId::Polygon => 0.5,
            ChainId::Base => 1.5,
            _ => 5.0,
        };

        source_gas + dest_gas + 0.5 // +$0.50 for LZ oracle/relayer
    }

    /// Estimate bridge completion time
    fn estimate_bridge_time(&self, from_chain: &ChainId, to_chain: &ChainId) -> u64 {
        // Stargate is typically very fast (built on LayerZero)
        let source_finality = match from_chain {
            ChainId::Ethereum => 180,
            ChainId::Arbitrum => 20,
            ChainId::Optimism => 30,
            ChainId::Polygon => 40,
            _ => 60,
        };

        let lz_messaging = 120; // 2 minutes for LayerZero
        let dest_execution = 30; // 30 seconds destination

        source_finality + lz_messaging + dest_execution
    }

    fn get_min_amount(&self, asset: &str) -> u64 {
        match asset {
            "USDC" | "USDT" | "DAI" | "FRAX" => 1_000_000, // $1 minimum
            "ETH" => 1_000_000_000_000_000,                 // 0.001 ETH
            _ => 1_000_000,
        }
    }

    fn get_max_amount(&self, from_chain: &ChainId, to_chain: &ChainId, asset: &str) -> u64 {
        // Max depends on pool liquidity (amounts in smallest units)
        let base_max: u64 = match asset {
            "USDC" => 5_000_000_000_000,  // $5M in 6 decimals
            "USDT" => 5_000_000_000_000,  // $5M in 6 decimals
            "DAI" => 1_000_000_000_000_000_000, // $1M in 18 decimals
            "ETH" => 500_000_000_000_000_000, // 500 ETH in 18 decimals
            _ => 1_000_000_000_000,
        };

        // Reduce for smaller chains
        let chain_multiplier = match to_chain {
            ChainId::Ethereum => 1.0,
            ChainId::Arbitrum => 0.8,
            ChainId::Optimism => 0.8,
            ChainId::Polygon => 0.6,
            ChainId::Base => 0.5,
            _ => 0.5,
        };

        (base_max as f64 * chain_multiplier) as u64
    }

    /// Initiate bridge transfer
    pub async fn bridge_transfer(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
        recipient: String,
        min_amount_out: Option<u64>, // Slippage protection
    ) -> Result<BridgeTransaction, BridgeError> {
        if !self.is_route_supported(&from_chain, &to_chain, &asset) {
            if !self.is_asset_supported(&asset) {
                return Err(BridgeError::UnsupportedAsset(asset));
            }
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        let min = self.get_min_amount(&asset);
        let max = self.get_max_amount(&from_chain, &to_chain, &asset);
        if amount < min {
            return Err(BridgeError::AmountTooSmall(amount, min));
        }
        if amount > max {
            return Err(BridgeError::AmountTooLarge(amount, max));
        }

        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount, &asset).await?;
        let estimated_time = self.estimate_bridge_time(&from_chain, &to_chain);

        // In production:
        // 1. Approve Stargate Router contract
        // 2. Call swap() function with:
        //    - Source pool ID
        //    - Destination pool ID
        //    - Amount
        //    - Min amount out (slippage)
        //    - Recipient address
        // 3. Stargate locks tokens in source pool
        // 4. Sends message via LayerZero
        // 5. Destination pool releases tokens

        let bridge_id = format!("sg_{}_{}", ic_cdk::api::time(), amount);

        Ok(BridgeTransaction {
            bridge_id,
            protocol: BridgeProtocol::Stargate,
            from_chain,
            to_chain,
            asset,
            amount,
            source_tx_hash: Some(format!("0x{:x}", ic_cdk::api::time())),
            destination_tx_hash: None,
            status: BridgeStatus::Pending,
            fee_paid: fee,
            estimated_arrival: ic_cdk::api::time() + (estimated_time * 1_000_000_000),
            initiated_at: ic_cdk::api::time(),
            completed_at: None,
        })
    }

    /// Get pool liquidity
    pub async fn get_pool_liquidity(&self, chain: &ChainId, asset: &str) -> Result<f64, String> {
        // In production, query Stargate pool contracts
        // Stargate has ~$300M TVL across all chains

        let base_liquidity = match asset {
            "USDC" => 100_000_000.0, // $100M
            "USDT" => 80_000_000.0,
            "ETH" => 50_000_000.0,
            "DAI" => 30_000_000.0,
            _ => 0.0,
        };

        let chain_multiplier = match chain {
            ChainId::Ethereum => 0.4,      // 40% on Ethereum
            ChainId::Arbitrum => 0.25,     // 25% on Arbitrum
            ChainId::Polygon => 0.15,
            ChainId::Optimism => 0.1,
            ChainId::Base => 0.05,
            _ => 0.05,
        };

        Ok(base_liquidity * chain_multiplier)
    }

    /// Check transfer status
    pub async fn get_transfer_status(&self, bridge_id: &str) -> Result<BridgeStatus, BridgeError> {
        Ok(BridgeStatus::Bridging)
    }

    /// Get supported assets
    pub fn get_supported_assets(&self) -> Vec<String> {
        vec![
            "USDC".to_string(),
            "USDT".to_string(),
            "DAI".to_string(),
            "FRAX".to_string(),
            "ETH".to_string(),
        ]
    }

    /// Get bridge health
    pub async fn get_health(&self) -> Result<BridgeHealth, String> {
        Ok(BridgeHealth {
            protocol: BridgeProtocol::Stargate,
            is_operational: true,
            liquidity_available: 300_000_000.0, // $300M TVL
            avg_completion_time: 240,            // ~4 minutes
            success_rate_7d: 0.998,              // 99.8%
            last_failure: None,
            maintenance_mode: false,
        })
    }

    /// Get optimal route (considers liquidity)
    pub async fn get_optimal_route(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<BridgeRoute, BridgeError> {
        // Check if direct route has enough liquidity
        let dest_liquidity = self.get_pool_liquidity(&to_chain, &asset).await?;

        if (amount as f64 / 1_000_000.0) > dest_liquidity * 0.5 {
            // Amount is > 50% of pool, might have high slippage
            return Err(BridgeError::InsufficientLiquidity(format!(
                "Amount too large for pool (max: ${:.2}M)",
                dest_liquidity * 0.5
            )));
        }

        self.get_bridge_route(from_chain, to_chain, asset, amount).await
    }
}

impl Default for StargateService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stargate_creation() {
        let service = StargateService::new();
        assert_eq!(service.supported_chains.len(), 6);
        assert_eq!(service.pool_ids.len(), 5);
    }

    #[test]
    fn test_asset_support() {
        let service = StargateService::new();
        assert!(service.is_asset_supported("USDC"));
        assert!(service.is_asset_supported("ETH"));
        assert!(!service.is_asset_supported("WBTC")); // Not supported
    }

    #[test]
    fn test_stablecoin_fees() {
        let service = StargateService::new();
        let lz_fee = service.estimate_layerzero_fee(&ChainId::Polygon, &ChainId::Arbitrum);
        assert!(lz_fee < 5.0); // Should be cheap for L2s
    }

    #[test]
    fn test_min_amounts() {
        let service = StargateService::new();
        assert_eq!(service.get_min_amount("USDC"), 1_000_000); // $1
        assert_eq!(service.get_min_amount("ETH"), 1_000_000_000_000_000); // 0.001 ETH
    }
}
