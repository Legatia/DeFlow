// Wormhole Bridge Integration
// Cross-chain messaging and token bridge protocol
// https://wormhole.com

use super::*;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,
};
use std::collections::HashMap;

const WORMHOLE_API_MAINNET: &str = "https://api.wormholescan.io";
const WORMHOLE_API_TESTNET: &str = "https://api.testnet.wormholescan.io";

/// Wormhole chain IDs (different from our internal ChainId)
const WORMHOLE_CHAIN_ETHEREUM: u16 = 2;
const WORMHOLE_CHAIN_BSC: u16 = 4;
const WORMHOLE_CHAIN_POLYGON: u16 = 5;
const WORMHOLE_CHAIN_AVALANCHE: u16 = 6;
const WORMHOLE_CHAIN_ARBITRUM: u16 = 23;
const WORMHOLE_CHAIN_OPTIMISM: u16 = 24;
const WORMHOLE_CHAIN_BASE: u16 = 30;
const WORMHOLE_CHAIN_SOLANA: u16 = 1;

/// Wormhole bridge service
#[derive(Debug, Clone)]
pub struct WormholeService {
    pub network: WormholeNetwork,
    pub supported_chains: HashMap<ChainId, u16>,
    pub http_timeout: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WormholeNetwork {
    Mainnet,
    Testnet,
}

impl WormholeNetwork {
    fn api_url(&self) -> &str {
        match self {
            WormholeNetwork::Mainnet => WORMHOLE_API_MAINNET,
            WormholeNetwork::Testnet => WORMHOLE_API_TESTNET,
        }
    }
}

impl WormholeService {
    pub fn new(network: WormholeNetwork) -> Self {
        let mut supported_chains = HashMap::new();
        supported_chains.insert(ChainId::Ethereum, WORMHOLE_CHAIN_ETHEREUM);
        supported_chains.insert(ChainId::Polygon, WORMHOLE_CHAIN_POLYGON);
        supported_chains.insert(ChainId::Arbitrum, WORMHOLE_CHAIN_ARBITRUM);
        supported_chains.insert(ChainId::Optimism, WORMHOLE_CHAIN_OPTIMISM);
        supported_chains.insert(ChainId::Base, WORMHOLE_CHAIN_BASE);
        supported_chains.insert(ChainId::Avalanche, WORMHOLE_CHAIN_AVALANCHE);
        supported_chains.insert(ChainId::Solana, WORMHOLE_CHAIN_SOLANA);

        Self {
            network,
            supported_chains,
            http_timeout: 30_000_000_000, // 30 seconds
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

        // Query Wormhole for current fees and availability
        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount).await?;
        let time = self.estimate_bridge_time(&from_chain, &to_chain);

        Ok(BridgeRoute {
            protocol: BridgeProtocol::Wormhole,
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            estimated_fee: fee,
            estimated_time_seconds: time,
            min_amount: self.get_min_amount(&asset),
            max_amount: self.get_max_amount(&asset),
            security_score: 0.85, // Wormhole has good security
            success_rate_24h: 0.98,
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
        // Wormhole fees consist of:
        // 1. Source chain gas
        // 2. Guardian network fee (VAA verification)
        // 3. Destination chain gas

        let source_gas = self.estimate_chain_gas(from_chain);
        let dest_gas = self.estimate_chain_gas(to_chain);
        let guardian_fee = 0.25; // ~$0.25 for VAA verification
        let protocol_fee = (amount as f64) * 0.0001; // 0.01% of amount

        Ok(source_gas + dest_gas + guardian_fee + protocol_fee)
    }

    /// Estimate chain-specific gas costs
    fn estimate_chain_gas(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Ethereum => 15.0,
            ChainId::Arbitrum => 2.0,
            ChainId::Optimism => 2.5,
            ChainId::Polygon => 0.5,
            ChainId::Base => 1.5,
            ChainId::Avalanche => 1.0,
            ChainId::Solana => 0.01,
            _ => 5.0,
        }
    }

    /// Estimate bridge completion time
    fn estimate_bridge_time(&self, from_chain: &ChainId, to_chain: &ChainId) -> u64 {
        // Wormhole uses Guardian network (~15-30 min finality)
        let source_finality = self.get_chain_finality(from_chain);
        let guardian_confirmation = 900; // 15 minutes for guardian signatures
        let dest_execution = self.get_chain_finality(to_chain);

        source_finality + guardian_confirmation + dest_execution
    }

    fn get_chain_finality(&self, chain: &ChainId) -> u64 {
        match chain {
            ChainId::Ethereum => 180,      // 3 minutes
            ChainId::Arbitrum => 60,       // 1 minute
            ChainId::Optimism => 120,      // 2 minutes
            ChainId::Polygon => 90,        // 1.5 minutes
            ChainId::Solana => 30,         // 30 seconds
            ChainId::Avalanche => 60,      // 1 minute
            _ => 120,
        }
    }

    /// Get minimum bridge amount
    fn get_min_amount(&self, asset: &str) -> u64 {
        match asset {
            "USDC" | "USDT" => 10_000_000,     // $10 in 6 decimals
            "ETH" | "WETH" => 5_000_000_000_000_000, // 0.005 ETH in wei
            "BTC" | "WBTC" => 10_000,          // 0.0001 BTC in sats
            _ => 1_000_000,                     // Default $1
        }
    }

    /// Get maximum bridge amount
    fn get_max_amount(&self, asset: &str) -> u64 {
        match asset {
            "USDC" | "USDT" => 1_000_000_000_000, // $1M in 6 decimals
            "ETH" | "WETH" => 100_000_000_000_000_000, // 100 ETH in 18 decimals
            "BTC" | "WBTC" => 100_000_000,     // 1 BTC in 8 decimals
            _ => 100_000_000_000,               // Default $100k
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
        // Validate route
        if !self.is_route_supported(&from_chain, &to_chain) {
            return Err(BridgeError::UnsupportedRoute(from_chain, to_chain));
        }

        // Validate amount
        let min = self.get_min_amount(&asset);
        let max = self.get_max_amount(&asset);
        if amount < min {
            return Err(BridgeError::AmountTooSmall(amount, min));
        }
        if amount > max {
            return Err(BridgeError::AmountTooLarge(amount, max));
        }

        // Calculate fee
        let fee = self.estimate_bridge_fee(&from_chain, &to_chain, amount).await?;
        let estimated_time = self.estimate_bridge_time(&from_chain, &to_chain);

        // In production, this would:
        // 1. Call Wormhole Core Bridge contract on source chain
        // 2. Lock/burn tokens
        // 3. Get VAA (Verified Action Approval) from guardians
        // 4. Submit VAA to destination chain
        // 5. Mint/unlock tokens on destination

        let bridge_id = format!("wh_{}_{}", ic_cdk::api::time(), amount);

        Ok(BridgeTransaction {
            bridge_id,
            protocol: BridgeProtocol::Wormhole,
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
        // In production, query Wormhole API for VAA status
        let url = format!("{}/api/v1/vaas/{}", self.network.api_url(), bridge_id);

        // Mock response for now
        Ok(BridgeStatus::Bridging)
    }

    /// Get supported assets for a chain
    pub fn get_supported_assets(&self, chain: &ChainId) -> Vec<String> {
        match chain {
            ChainId::Ethereum => vec![
                "USDC".to_string(),
                "USDT".to_string(),
                "ETH".to_string(),
                "WETH".to_string(),
                "WBTC".to_string(),
            ],
            ChainId::Solana => vec![
                "USDC".to_string(),
                "USDT".to_string(),
                "SOL".to_string(),
            ],
            _ => vec![
                "USDC".to_string(),
                "USDT".to_string(),
            ],
        }
    }

    /// Get total value locked in Wormhole
    pub async fn get_tvl(&self) -> Result<f64, String> {
        let url = format!("{}/api/v1/totals", self.network.api_url());

        // In production, parse actual API response
        // Wormhole TVL is ~$500M as of 2025
        Ok(500_000_000.0)
    }

    /// Get bridge health metrics
    pub async fn get_health(&self) -> Result<BridgeHealth, String> {
        // In production, query actual Wormhole metrics
        Ok(BridgeHealth {
            protocol: BridgeProtocol::Wormhole,
            is_operational: true,
            liquidity_available: 500_000_000.0, // $500M TVL
            avg_completion_time: 1200,           // 20 minutes
            success_rate_7d: 0.985,              // 98.5%
            last_failure: None,
            maintenance_mode: false,
        })
    }

    /// HTTP request helper
    async fn http_get(&self, url: &str) -> Result<String, String> {
        let request = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(10_000),
            transform: None,
            headers: vec![],
        };

        match http_request(request, self.http_timeout as u128).await {
            Ok((response,)) => {
                if response.status == 200u8 {
                    String::from_utf8(response.body)
                        .map_err(|e| format!("Failed to parse response: {}", e))
                } else {
                    Err(format!("HTTP error: {}", response.status))
                }
            }
            Err((code, msg)) => Err(format!("HTTP request failed: {:?} - {}", code, msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wormhole_service_creation() {
        let service = WormholeService::new(WormholeNetwork::Mainnet);
        assert_eq!(service.supported_chains.len(), 7);
        assert!(service.is_route_supported(&ChainId::Ethereum, &ChainId::Polygon));
    }

    #[test]
    fn test_unsupported_route() {
        let service = WormholeService::new(WormholeNetwork::Mainnet);
        assert!(!service.is_route_supported(&ChainId::Bitcoin, &ChainId::Ethereum));
    }

    #[test]
    fn test_min_max_amounts() {
        let service = WormholeService::new(WormholeNetwork::Mainnet);

        let min_usdc = service.get_min_amount("USDC");
        let max_usdc = service.get_max_amount("USDC");

        assert_eq!(min_usdc, 10_000_000);          // $10
        assert_eq!(max_usdc, 1_000_000_000_000);   // $1M
    }

    #[test]
    fn test_chain_gas_estimates() {
        let service = WormholeService::new(WormholeNetwork::Mainnet);

        assert_eq!(service.estimate_chain_gas(&ChainId::Ethereum), 15.0);
        assert_eq!(service.estimate_chain_gas(&ChainId::Solana), 0.01);
        assert_eq!(service.estimate_chain_gas(&ChainId::Polygon), 0.5);
    }
}
