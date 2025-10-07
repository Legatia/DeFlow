// L2 Bridge Executor - Real contract interactions for Arbitrum, Optimism, Base bridges
// Uses ICP EVM RPC to call official bridge contracts

use super::yield_farming::ChainId;
use super::ethereum::minimal_icp::MinimalIcpEthereumService;
use candid::Principal;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

/// Official L2 bridge contracts (Mainnet)
pub struct L2BridgeContracts;

impl L2BridgeContracts {
    // Arbitrum Bridge
    pub const ARBITRUM_BRIDGE: &'static str = "0x4Dbd4fc535Ac27206064B68FfCf827b0A60BAB3f";
    pub const ARBITRUM_ERC20_GATEWAY: &'static str = "0xa3A7B6F88361F48403514059F1F16C8E78d60EeC";
    
    // Optimism Bridge  
    pub const OPTIMISM_BRIDGE: &'static str = "0x99C9fc46f92E8a1c0deC1b1747d010903E884bE1";
    pub const OPTIMISM_ERC20_BRIDGE: &'static str = "0x5a7749f83b81B301cAb5f48EB8516B986DAef23D";
    
    // Base Bridge
    pub const BASE_BRIDGE: &'static str = "0x49048044D57e1C92A77f79988d21Fa8fAF74E97e";
    pub const BASE_ERC20_BRIDGE: &'static str = "0x3154Cf16ccdb4C6d922629664174b904d80F2C35";
    
    // Polygon PoS Bridge
    pub const POLYGON_BRIDGE: &'static str = "0xA0c68C638235ee32657e8f720a23ceC1bFc77C77";
}

/// RPC endpoints for different chains
pub struct ChainRpcEndpoints;

impl ChainRpcEndpoints {
    pub fn get_rpc(chain: &ChainId) -> &'static str {
        match chain {
            ChainId::Ethereum => "https://eth.llamarpc.com",
            ChainId::Arbitrum => "https://arb1.arbitrum.io/rpc",
            ChainId::Optimism => "https://mainnet.optimism.io",
            ChainId::Base => "https://mainnet.base.org",
            ChainId::Polygon => "https://polygon-rpc.com",
            _ => "https://eth.llamarpc.com",
        }
    }
}

/// L2 Bridge Executor
#[derive(Debug, Clone)]
pub struct L2BridgeExecutor {
    pub eth_service: MinimalIcpEthereumService,
}

impl L2BridgeExecutor {
    pub fn new(canister_id: Principal) -> Self {
        Self {
            eth_service: MinimalIcpEthereumService::new(
                "deflow_bridge_key".to_string(),
                canister_id
            ),
        }
    }

    /// Bridge ETH from Ethereum to L2
    pub async fn bridge_eth_to_l2(
        &self,
        to_chain: &ChainId,
        amount_wei: u128,
        user_address: &str,
    ) -> Result<String, String> {
        let (bridge_contract, method_signature) = match to_chain {
            ChainId::Arbitrum => (
                L2BridgeContracts::ARBITRUM_BRIDGE,
                "depositEth()"  // Arbitrum bridge method
            ),
            ChainId::Optimism => (
                L2BridgeContracts::OPTIMISM_BRIDGE,
                "depositETH(uint32 _minGasLimit, bytes calldata _extraData)"
            ),
            ChainId::Base => (
                L2BridgeContracts::BASE_BRIDGE,
                "depositETH(uint32 _minGasLimit, bytes calldata _extraData)"
            ),
            _ => return Err("Unsupported L2 chain".to_string()),
        };

        // Encode the transaction data
        let tx_data = self.encode_bridge_eth_call(to_chain, amount_wei)?;

        // Execute via EVM RPC
        let tx_hash = self.execute_contract_call(
            &ChainId::Ethereum,
            bridge_contract,
            &tx_data,
            amount_wei,
        ).await?;

        Ok(tx_hash)
    }

    /// Bridge ERC20 tokens to L2
    pub async fn bridge_erc20_to_l2(
        &self,
        to_chain: &ChainId,
        token_address: &str,
        amount: u128,
        user_address: &str,
    ) -> Result<String, String> {
        let (gateway_contract, method) = match to_chain {
            ChainId::Arbitrum => (
                L2BridgeContracts::ARBITRUM_ERC20_GATEWAY,
                "outboundTransfer"
            ),
            ChainId::Optimism => (
                L2BridgeContracts::OPTIMISM_ERC20_BRIDGE,
                "depositERC20"
            ),
            ChainId::Base => (
                L2BridgeContracts::BASE_ERC20_BRIDGE,
                "depositERC20"
            ),
            _ => return Err("Unsupported L2 chain".to_string()),
        };

        // Step 1: Approve gateway to spend tokens
        let approve_hash = self.approve_erc20(
            token_address,
            gateway_contract,
            amount,
        ).await?;

        ic_cdk::println!("ERC20 approved: {}", approve_hash);

        // Step 2: Execute bridge transfer
        let tx_data = self.encode_bridge_erc20_call(
            to_chain,
            token_address,
            amount,
            user_address,
        )?;

        let tx_hash = self.execute_contract_call(
            &ChainId::Ethereum,
            gateway_contract,
            &tx_data,
            0, // No ETH value for ERC20 bridge
        ).await?;

        Ok(tx_hash)
    }

    /// Execute contract call via EVM RPC
    async fn execute_contract_call(
        &self,
        chain: &ChainId,
        contract_address: &str,
        data: &str,
        value_wei: u128,
    ) -> Result<String, String> {
        let rpc_url = ChainRpcEndpoints::get_rpc(chain);

        // Build eth_sendTransaction JSON-RPC request
        let request_body = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "eth_sendTransaction",
                "params": [{{
                    "to": "{}",
                    "data": "{}",
                    "value": "0x{:x}",
                    "gas": "0x100000"
                }}],
                "id": 1
            }}"#,
            contract_address,
            data,
            value_wei
        );

        // Make HTTP outcall
        let request = CanisterHttpRequestArgument {
            url: rpc_url.to_string(),
            method: HttpMethod::POST,
            body: Some(request_body.as_bytes().to_vec()),
            max_response_bytes: Some(2000),
            transform: None,
            headers: vec![
                HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
        };

        match http_request(request, 25_000_000_000).await {
            Ok((response,)) => {
                let body = String::from_utf8(response.body)
                    .map_err(|e| format!("Invalid UTF8 response: {}", e))?;
                
                // Parse tx hash from JSON response
                if let Some(tx_hash) = self.extract_tx_hash(&body) {
                    Ok(tx_hash)
                } else {
                    Err(format!("Failed to extract tx hash from: {}", body))
                }
            },
            Err((code, msg)) => {
                Err(format!("HTTP request failed: {:?} - {}", code, msg))
            }
        }
    }

    /// Approve ERC20 token spending
    async fn approve_erc20(
        &self,
        token_address: &str,
        spender: &str,
        amount: u128,
    ) -> Result<String, String> {
        // ERC20 approve function signature: approve(address spender, uint256 amount)
        // Function selector: 0x095ea7b3
        let data = format!(
            "0x095ea7b3{:0>64}{:0>64}",
            &spender[2..],  // Remove 0x prefix and pad to 32 bytes
            format!("{:x}", amount)
        );

        self.execute_contract_call(
            &ChainId::Ethereum,
            token_address,
            &data,
            0,
        ).await
    }

    /// Encode depositEth call for different L2s
    fn encode_bridge_eth_call(&self, chain: &ChainId, amount: u128) -> Result<String, String> {
        match chain {
            ChainId::Arbitrum => {
                // depositEth() - no parameters
                Ok("0x0f4d14e9".to_string())
            },
            ChainId::Optimism | ChainId::Base => {
                // depositETH(uint32 _minGasLimit, bytes calldata _extraData)
                // _minGasLimit = 200000, _extraData = empty
                Ok(format!(
                    "0x{:0>8}{:0>64}{:0>64}",
                    "b1a1a882",  // Function selector
                    "00030d40",  // 200000 in hex
                    "40"         // Offset to _extraData
                ))
            },
            _ => Err("Unsupported chain".to_string()),
        }
    }

    /// Encode ERC20 bridge call
    fn encode_bridge_erc20_call(
        &self,
        chain: &ChainId,
        token: &str,
        amount: u128,
        recipient: &str,
    ) -> Result<String, String> {
        match chain {
            ChainId::Arbitrum => {
                // outboundTransfer(address _token, address _to, uint256 _amount, uint256 _maxGas, uint256 _gasPriceBid, bytes calldata _data)
                Ok(format!(
                    "0x{:0>8}{:0>64}{:0>64}{:0>64}{:0>64}{:0>64}{:0>64}",
                    "7b3a3c8b",  // Function selector
                    &token[2..],
                    &recipient[2..],
                    format!("{:x}", amount),
                    "100000",    // maxGas
                    "100000000", // gasPriceBid (0.1 gwei)
                    "c0"         // data offset (empty)
                ))
            },
            ChainId::Optimism | ChainId::Base => {
                // depositERC20(address _l1Token, address _l2Token, uint256 _amount, uint32 _minGasLimit, bytes calldata _extraData)
                Ok(format!(
                    "0x{:0>8}{:0>64}{:0>64}{:0>64}{:0>64}{:0>64}",
                    "58a997f6",  // Function selector
                    &token[2..],
                    &token[2..],  // L2 token same as L1
                    format!("{:x}", amount),
                    "200000",     // minGasLimit
                    "a0"          // extraData offset (empty)
                ))
            },
            _ => Err("Unsupported chain".to_string()),
        }
    }

    /// Extract transaction hash from JSON-RPC response
    fn extract_tx_hash(&self, json: &str) -> Option<String> {
        // Simple JSON parsing - in production use serde_json
        if let Some(start) = json.find("\"result\":\"") {
            let start = start + 10; // length of "\"result\":\""
            if let Some(end) = json[start..].find("\"") {
                return Some(json[start..start + end].to_string());
            }
        }
        None
    }

    /// Get bridge fee estimate
    pub async fn estimate_bridge_fee(&self, to_chain: &ChainId) -> Result<f64, String> {
        // Get current gas price
        let gas_price = self.get_gas_price(&ChainId::Ethereum).await?;
        
        // Estimate gas usage based on L2
        let gas_units: u64 = match to_chain {
            ChainId::Arbitrum => 150_000,  // Arbitrum deposits use ~150k gas
            ChainId::Optimism => 200_000,  // Optimism uses ~200k gas
            ChainId::Base => 200_000,      // Base similar to Optimism
            ChainId::Polygon => 100_000,   // Polygon PoS bridge cheaper
            _ => 200_000,
        };

        // Calculate fee in USD (assuming ETH = $2000)
        let fee_eth = (gas_price * gas_units as f64) / 1e18;
        let fee_usd = fee_eth * 2000.0;

        Ok(fee_usd)
    }

    /// Get current gas price from chain
    async fn get_gas_price(&self, chain: &ChainId) -> Result<f64, String> {
        let rpc_url = ChainRpcEndpoints::get_rpc(chain);

        let request_body = r#"{"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":1}"#;

        let request = CanisterHttpRequestArgument {
            url: rpc_url.to_string(),
            method: HttpMethod::POST,
            body: Some(request_body.as_bytes().to_vec()),
            max_response_bytes: Some(1000),
            transform: None,
            headers: vec![
                HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
        };

        match http_request(request, 25_000_000_000).await {
            Ok((response,)) => {
                let body = String::from_utf8_lossy(&response.body);
                // Extract gas price from JSON (in wei)
                if let Some(gas_price_hex) = self.extract_hex_value(&body) {
                    let gas_price = u64::from_str_radix(&gas_price_hex[2..], 16)
                        .map_err(|e| format!("Failed to parse gas price: {}", e))?;
                    Ok(gas_price as f64)
                } else {
                    Ok(50_000_000_000.0) // Default 50 gwei
                }
            },
            Err(_) => Ok(50_000_000_000.0), // Default 50 gwei on error
        }
    }

    fn extract_hex_value(&self, json: &str) -> Option<String> {
        if let Some(start) = json.find("\"result\":\"") {
            let start = start + 10;
            if let Some(end) = json[start..].find("\"") {
                return Some(json[start..start + end].to_string());
            }
        }
        None
    }
}

// Global instance
use std::cell::RefCell;
thread_local! {
    static L2_BRIDGE_EXECUTOR: RefCell<Option<L2BridgeExecutor>> = RefCell::new(None);
}

pub fn init_l2_bridge_executor(canister_id: Principal) {
    L2_BRIDGE_EXECUTOR.with(|executor| {
        *executor.borrow_mut() = Some(L2BridgeExecutor::new(canister_id));
    });
}

pub fn with_l2_bridge_executor<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&L2BridgeExecutor) -> R,
{
    L2_BRIDGE_EXECUTOR.with(|executor| {
        executor.borrow().as_ref().map(f)
    })
}
