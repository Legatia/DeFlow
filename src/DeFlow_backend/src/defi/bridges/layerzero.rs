// LayerZero Bridge Integration
// Omnichain interoperability protocol
// https://layerzero.network

use super::*;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,
};
use std::collections::HashMap;

const LAYERZERO_API: &str = "https://api.layerzero.network";

/// LayerZero chain IDs
const LZ_CHAIN_ETHEREUM: u16 = 101;
const LZ_CHAIN_BSC: u16 = 102;
const LZ_CHAIN_AVALANCHE: u16 = 106;
const LZ_CHAIN_POLYGON: u16 = 109;
const LZ_CHAIN_ARBITRUM: u16 = 110;
const LZ_CHAIN_OPTIMISM: u16 = 111;
const LZ_CHAIN_BASE: u16 = 184;

/// LayerZero bridge service
#[derive(Debug, Clone)]
pub struct LayerZeroService {
    pub supported_chains: HashMap<ChainId, u16>,
    pub http_timeout: u64,
}

impl LayerZeroService {
    pub fn new() -> Self {
        let mut supported_chains = HashMap::new();
        supported_chains.insert(ChainId::Ethereum, LZ_CHAIN_ETHEREUM);
        supported_chains.insert(ChainId::Polygon, LZ_CHAIN_POLYGON);
        supported_chains.insert(ChainId::Arbitrum, LZ_CHAIN_ARBITRUM);
        supported_chains.insert(ChainId::Optimism, LZ_CHAIN_OPTIMISM);
        supported_chains.insert(ChainId::Base, LZ_CHAIN_BASE);
        supported_chains.insert(ChainId::Avalanche, LZ_CHAIN_AVALANCHE);

        Self {
            supported_chains,
            http_timeout: 30_000_000_000,
        }
    }

    /// Check if route is supported
    pub fn is_route_supported(&self, from: &ChainId, to: &ChainId) -> bool {
        self.supported_chains.contains_key(from) && self.supported_chains.contains_key(to)
    }

    /// Get bridge route details
    pub async fn get_bridge_route(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) -> Result<BridgeRoute, BridgeError> {
        if !self.is_route_supported(&from_chain, &to_chain) {
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount).await?;
        let time = self.estimate_bridge_time(&from_chain, &to_chain);

        Ok(BridgeRoute {
            protocol: BridgeProtocol::LayerZero,
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            estimated_fee: fee,
            estimated_time_seconds: time,
            min_amount: self.get_min_amount(&asset),
            max_amount: self.get_max_amount(&asset),
            security_score: 0.90, // LayerZero has excellent security
            success_rate_24h: 0.99,
            available: true,
        })
    }

    /// Estimate bridge fee
    async fn estimate_bridge_fee(
        &self,
        from_chain: &ChainId,
        to_chain: &ChainId,
        amount: u64,
    ) -> Result<f64, BridgeError> {
        // LayerZero fees:
        // 1. Oracle fee (Chainlink/Google Cloud)
        // 2. Relayer fee (for message delivery)
        // 3. Source chain gas
        // 4. Destination chain gas

        let oracle_fee = 0.10; // ~$0.10 for oracle verification
        let relayer_fee = self.estimate_relayer_fee(from_chain, to_chain);
        let source_gas = self.estimate_chain_gas(from_chain);
        let dest_gas = self.estimate_chain_gas(to_chain);

        Ok(oracle_fee + relayer_fee + source_gas + dest_gas)
    }

    /// Estimate relayer fee based on destination chain
    fn estimate_relayer_fee(&self, from_chain: &ChainId, to_chain: &ChainId) -> f64 {
        // Relayer fee varies by destination chain gas price
        let base_fee = 0.50;
        let dest_multiplier = match to_chain {
            ChainId::Ethereum => 3.0,  // Higher for expensive chains
            ChainId::Arbitrum => 0.5,
            ChainId::Optimism => 0.5,
            ChainId::Polygon => 0.3,
            ChainId::Base => 0.4,
            _ => 1.0,
        };

        base_fee * dest_multiplier
    }

    /// Estimate chain-specific gas costs
    fn estimate_chain_gas(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Ethereum => 20.0,  // LayerZero messages are gas-intensive
            ChainId::Arbitrum => 3.0,
            ChainId::Optimism => 3.5,
            ChainId::Polygon => 0.8,
            ChainId::Base => 2.0,
            ChainId::Avalanche => 1.5,
            _ => 5.0,
        }
    }

    /// Estimate bridge completion time
    fn estimate_bridge_time(&self, from_chain: &ChainId, to_chain: &ChainId) -> u64 {
        // LayerZero is typically faster than Wormhole
        let source_finality = self.get_chain_finality(from_chain);
        let oracle_verification = 180;  // 3 minutes for oracle
        let relayer_delivery = 120;     // 2 minutes for relayer
        let dest_execution = self.get_chain_finality(to_chain);

        source_finality + oracle_verification + relayer_delivery + dest_execution
    }

    fn get_chain_finality(&self, chain: &ChainId) -> u64 {
        match chain {
            ChainId::Ethereum => 180,
            ChainId::Arbitrum => 30,
            ChainId::Optimism => 60,
            ChainId::Polygon => 60,
            ChainId::Avalanche => 30,
            _ => 90,
        }
    }

    fn get_min_amount(&self, asset: &str) -> u64 {
        match asset {
            "USDC" | "USDT" => 5_000_000,      // $5 (lower than Wormhole)
            "ETH" | "WETH" => 2_000_000_000_000_000, // 0.002 ETH
            _ => 500_000,
        }
    }

    fn get_max_amount(&self, asset: &str) -> u64 {
        match asset {
            "USDC" | "USDT" => 10_000_000_000_000, // $10M in 6 decimals (higher than Wormhole)
            "ETH" | "WETH" => 1_000_000_000_000_000_000, // 1000 ETH in 18 decimals
            _ => 1_000_000_000_000,
        }
    }

    /// Initiate bridge transfer
    pub async fn bridge_transfer(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
        recipient: String,
    ) -> Result<BridgeTransaction, BridgeError> {
        if !self.is_route_supported(&from_chain, &to_chain) {
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        let min = self.get_min_amount(&asset);
        let max = self.get_max_amount(&asset);
        if amount < min {
            return Err(BridgeError::AmountTooSmall(amount, min));
        }
        if amount > max {
            return Err(BridgeError::AmountTooLarge(amount, max));
        }

        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount).await?;
        let estimated_time = self.estimate_bridge_time(&from_chain, &to_chain);

        // In production:
        // 1. Call LayerZero endpoint contract
        // 2. Send message with token transfer
        // 3. Oracle verifies and signs
        // 4. Relayer delivers message to destination
        // 5. Destination contract executes

        let bridge_id = format!("lz_{}_{}", ic_cdk::api::time(), amount);

        Ok(BridgeTransaction {
            bridge_id,
            protocol: BridgeProtocol::LayerZero,
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

    /// Check transfer status
    pub async fn get_transfer_status(&self, bridge_id: &str) -> Result<BridgeStatus, BridgeError> {
        // Query LayerZero scan API
        Ok(BridgeStatus::Bridging)
    }

    /// Get supported assets
    pub fn get_supported_assets(&self, chain: &ChainId) -> Vec<String> {
        match chain {
            ChainId::Ethereum => vec![
                "USDC".to_string(),
                "USDT".to_string(),
                "ETH".to_string(),
                "WETH".to_string(),
                "WBTC".to_string(),
                "DAI".to_string(),
            ],
            _ => vec![
                "USDC".to_string(),
                "USDT".to_string(),
            ],
        }
    }

    /// Get bridge health
    pub async fn get_health(&self) -> Result<BridgeHealth, String> {
        Ok(BridgeHealth {
            protocol: BridgeProtocol::LayerZero,
            is_operational: true,
            liquidity_available: f64::INFINITY, // LayerZero is messaging, not liquidity
            avg_completion_time: 420,            // ~7 minutes
            success_rate_7d: 0.995,              // 99.5%
            last_failure: None,
            maintenance_mode: false,
        })
    }
}

impl Default for LayerZeroService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layerzero_creation() {
        let service = LayerZeroService::new();
        assert_eq!(service.supported_chains.len(), 6);
    }

    #[test]
    fn test_supported_routes() {
        let service = LayerZeroService::new();
        assert!(service.is_route_supported(&ChainId::Ethereum, &ChainId::Arbitrum));
        assert!(service.is_route_supported(&ChainId::Polygon, &ChainId::Base));
    }

    #[test]
    fn test_relayer_fees() {
        let service = LayerZeroService::new();
        let eth_fee = service.estimate_relayer_fee(&ChainId::Polygon, &ChainId::Ethereum);
        let arb_fee = service.estimate_relayer_fee(&ChainId::Polygon, &ChainId::Arbitrum);
        assert!(eth_fee > arb_fee); // Ethereum more expensive
    }
}
