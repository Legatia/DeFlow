# ICP Chain Fusion: Multi-Chain Integration Guide

This guide documents how to interact with Bitcoin, Solana, and Ethereum blockchains through Internet Computer Protocol (ICP) native chain fusion technology.

## Overview

ICP's chain fusion enables canisters (smart contracts) to:
- Generate addresses for external chains
- Send and receive native tokens (BTC, SOL, ETH)
- Interact with blockchain-specific protocols
- Create multi-chain applications with unified logic

## Core Technologies

All three integrations leverage two fundamental ICP capabilities:

1. **Threshold Cryptography**:
   - Bitcoin: Threshold ECDSA + Threshold Schnorr (for Taproot)
   - Ethereum: Threshold ECDSA
   - Solana: Threshold EdDSA

2. **HTTPS Outcalls**: Direct communication with external blockchain nodes

## Bitcoin Integration

### Architecture
- Uses Bitcoin API, ECDSA API, and Schnorr API
- Supports legacy (P2PKH), SegWit (P2WPKH), and Taproot (P2TR) addresses  
- Includes Bitcoin assets: Ordinals, Runes, and BRC-20 tokens

### Key Implementation Patterns
```rust
// Context setup
struct BitcoinContext {
    network: Network,           // ICP Bitcoin API network
    bitcoin_network: bitcoin::Network, // Bitcoin crate network  
    key_name: &'static str,    // ECDSA key name
}
```

### Core Operations
1. **Address Generation**: 
   - `get_p2pkh_address()` - Legacy addresses
   - `get_p2wpkh_address()` - SegWit addresses  
   - `get_p2tr_key_path_only_address()` - Taproot key-path
   - `get_p2tr_script_path_enabled_address()` - Taproot script-path

2. **Balance Checking**: `get_balance(address)` using `bitcoin_get_balance`

3. **Sending Bitcoin**: Multiple endpoints for different address types
   - Estimates fees, looks up UTXOs, builds transaction
   - Signs with ECDSA/Schnorr, broadcasts with `bitcoin_send_transaction`

4. **Bitcoin Assets**:
   - **Ordinals**: Inscribe data onto satoshis via commit/reveal process
   - **Runes**: Fungible tokens using `OP_RETURN` outputs
   - **BRC-20**: JSON-based tokens built on Ordinals

### Deployment
```bash
dfx deploy basic_bitcoin --argument '(variant { regtest })'
```

## Ethereum Integration

### Architecture
- Uses threshold ECDSA and HTTPS outcalls
- Integrates with EVM-RPC canister for blockchain communication
- Supports EIP-1559 transactions

### Key Implementation Patterns
```rust
pub const EVM_RPC_CANISTER_ID: Principal = Principal::from_slice(...);
pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(EVM_RPC_CANISTER_ID);
```

### Core Operations
1. **Address Generation**: `ethereum_address(owner)` - Derives address from ECDSA public key

2. **Balance Checking**: `get_balance(address)` via `eth_getBalance` RPC call

3. **Transaction Count**: `transaction_count(owner, block)` for nonce management

4. **Sending ETH**: `send_eth(to, amount)`
   - Retrieves transaction count for nonce
   - Estimates gas fees (hardcoded for simplicity)
   - Builds EIP-1559 transaction
   - Signs with threshold ECDSA
   - Sends via `eth_sendRawTransaction`

### Network Support
- Mainnet and Sepolia testnet
- Configurable RPC services through EVM-RPC canister

### Deployment
```bash
dfx deploy --ic basic_ethereum --argument '(opt record {
  ethereum_network = opt variant {Sepolia}; 
  ecdsa_key_name = opt variant {TestKey1}
})'
```

## Solana Integration

### Architecture
- Uses threshold EdDSA and HTTPS outcalls
- Leverages SOL RPC canister for blockchain communication
- Supports Solana Program Library (SPL) tokens

### Core Operations
1. **Account Generation**: Solana accounts derived from EdDSA public key

2. **Token Operations**:
   - Send and receive SOL tokens
   - Create and manage associated token accounts
   - Send SPL tokens between accounts

3. **Transaction Building**:
   - Get recent blockhash
   - Build Solana transaction
   - Sign with threshold Ed25519
   - Send via SOL RPC canister

### Advanced Features
- Support for durable nonces
- Associated token account management
- SPL token interactions

## Common Integration Patterns

### 1. Key Derivation
All chains use deterministic key derivation based on:
- Canister's identity
- User's principal 
- Chain-specific derivation paths

### 2. Transaction Flow
1. **Preparation**: Get network state (nonce, fees, blockhash)
2. **Construction**: Build chain-specific transaction
3. **Signing**: Use appropriate threshold signature (ECDSA/EdDSA/Schnorr)
4. **Broadcasting**: Send via HTTPS outcalls to RPC nodes

### 3. State Management
```rust
thread_local! {
    static CONTEXT: Cell<ChainContext> = const { Cell::new(...) };
}
```

### 4. Error Handling
- Network failures gracefully handled
- Transaction validation before signing
- Comprehensive error reporting

## Security Considerations

### Best Practices for All Chains
1. **Decentralized Governance**: Use SNS for canister control
2. **Query Certification**: Certify security-relevant responses
3. **Key Management**: Proper derivation paths and key caching
4. **Cost Optimization**: Use testing canisters for development

### Chain-Specific Security
- **Bitcoin**: Manual transaction construction, fee estimation
- **Ethereum**: EIP-1559 compliance, gas limit management  
- **Solana**: Durable nonces, account validation

## Development Workflow

### Local Testing
1. **Bitcoin**: Start regtest network with `bitcoind`
2. **Ethereum**: Use Sepolia testnet or local fork
3. **Solana**: Connect to Devnet or local validator

### Production Deployment
1. Acquire cycles for canister deployment
2. Configure appropriate network (mainnet/testnet)
3. Set correct key names (test vs production)
4. Deploy with network-specific arguments

## Advanced Features

### Multi-Chain Applications
- Single canister can manage multiple chain integrations
- Unified user experience across chains
- Cross-chain atomic operations possible

### Asset Management
- **Bitcoin**: Full Bitcoin asset protocol support
- **Ethereum**: ERC-20 and other token standards
- **Solana**: SPL token ecosystem integration

## Resources

- Bitcoin Integration: `/rust/basic_bitcoin/`
- Ethereum Integration: `/rust/basic_ethereum/`  
- Solana Integration: [SOL RPC Canister](https://github.com/dfinity/sol-rpc-canister)
- [ICP Chain Fusion Documentation](https://internetcomputer.org/docs/current/developer-docs/multi-chain/)

## Future Reference

This guide provides the foundational patterns for implementing ICP chain fusion with Bitcoin, Ethereum, and Solana. For production applications:

1. Study the security considerations carefully
2. Implement proper error handling and retry logic
3. Use appropriate testing strategies for each chain
4. Consider decentralized governance from the start
5. Plan for scalability and cost optimization

The examples demonstrate basic functionality - production applications should extend these patterns with additional features like multi-signature support, advanced fee management, and comprehensive transaction history tracking.

```
Chain Fusion on the Internet Computer (ICP) enables native integration with Bitcoin, Ethereum, and Solana. For Layer 2 (L2) solutions, the situation is as follows:

Ethereum L2s and EVM Chains: ICP’s integration with Ethereum is built on the EVM RPC canister, which supports not only Ethereum mainnet but also other EVM-compatible networks and Layer 2 solutions (such as Optimism, Arbitrum, Polygon, and Avalanche). This means canisters can interact with these L2s in the same way as with Ethereum mainnet—querying state and submitting transactions—using HTTPS outcalls to multiple JSON-RPC providers. The integration is already live and production-ready for these networks How Does ICP Work? > Chain Fusion > Ethereum Integration Chain Fusion: Connecting Blockchains.

Bitcoin L2s: The current native integration is with the Bitcoin mainnet. There is no explicit mention in the knowledge sources of direct integration with Bitcoin L2s (such as Lightning Network) via Chain Fusion. However, ICP provides its own Layer 2 solution for Bitcoin in the form of ckBTC, which offers fast and cheap transfers programmable by canisters, but this is not the same as integrating with external Bitcoin L2s Bitcoin integration and ckBTC FAQ.

Solana L2s: The knowledge sources do not mention support for Solana Layer 2 solutions. The current integration is with the Solana mainnet.
Summary:
ICP’s Chain Fusion natively supports Ethereum L2s and EVM-compatible chains via the EVM RPC canister. For Bitcoin and Solana, the integration is currently with their mainnets, and there is no explicit support for their L2s mentioned in the available sources. If you need L2 support for Bitcoin or Solana, you may need to monitor future roadmap updates.

```
ICP’s Chain Fusion supports other EVM-compatible chains in the same way as Ethereum, through the EVM RPC canister. The documentation explicitly states that EVM-compatible chains other than Ethereum are supported via the EVM RPC canister, even if they are not listed individually. This means that as long as the EVM chain (such as Sonic) provides a compatible JSON-RPC endpoint and uses a supported authentication method (like ECDSA), it can be integrated with ICP using the same workflow as Ethereum Supported chains Supported chains.

You can use the EVM RPC canister’s request method to interact with many providers listed on ChainList.org, which includes a wide range of EVM-compatible networks. If Sonic is EVM-compatible and provides a public RPC endpoint, it should be supported by ICP’s Chain Fusion technology

```

