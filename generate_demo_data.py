#!/usr/bin/env python3
"""
DeFlow Yield Optimization Demo Data Generator

This script generates realistic 90-day CSV data for the yield optimization demo.
Includes realistic APY ranges, gas costs, and TVL values for various protocols and chains.
"""

import csv
import random
import math
from datetime import datetime, timedelta

# Configuration
START_DATE = datetime(2024, 7, 1)
DAYS = 90

# Protocol configurations with realistic APY ranges and TVL
PROTOCOLS = {
    "Aave": {"base_apy": 4.0, "variance": 1.5, "base_tvl": 250_000_000, "risk_factor": 0.8},
    "Compound": {"base_apy": 3.8, "variance": 1.2, "base_tvl": 180_000_000, "risk_factor": 0.7},
    "Curve": {"base_apy": 6.5, "variance": 2.5, "base_tvl": 320_000_000, "risk_factor": 1.2},
    "Uniswap V3": {"base_apy": 7.2, "variance": 3.0, "base_tvl": 95_000_000, "risk_factor": 1.4},
    "Yearn": {"base_apy": 5.1, "variance": 1.8, "base_tvl": 120_000_000, "risk_factor": 0.9},
    "Pendle": {"base_apy": 9.5, "variance": 4.0, "base_tvl": 75_000_000, "risk_factor": 1.8},
    "Convex": {"base_apy": 5.8, "variance": 2.0, "base_tvl": 150_000_000, "risk_factor": 1.0},
    "Balancer": {"base_apy": 6.8, "variance": 2.2, "base_tvl": 110_000_000, "risk_factor": 1.1},
}

# Chain configurations - REALISTIC COSTS FOR DEMO
CHAINS = {
    "Ethereum": {"gas_base": 15, "gas_variance": 8, "bridge_fee": 0, "apy_multiplier": 0.9},
    "Arbitrum": {"gas_base": 2, "gas_variance": 1, "bridge_fee": 5, "apy_multiplier": 1.1},
    "Optimism": {"gas_base": 1.5, "gas_variance": 0.8, "bridge_fee": 4, "apy_multiplier": 1.05},
    "Polygon": {"gas_base": 0.8, "gas_variance": 0.4, "bridge_fee": 3, "apy_multiplier": 1.15},
    "Base": {"gas_base": 1.2, "gas_variance": 0.6, "bridge_fee": 4, "apy_multiplier": 1.08},
}

STABLECOINS = ["USDC", "USDT", "DAI"]

def generate_apy(protocol, chain, day, base_volatility=0.1):
    """Generate realistic APY with market cycles and volatility"""
    protocol_config = PROTOCOLS[protocol]
    chain_config = CHAINS[chain]
    
    # Base APY
    base_apy = protocol_config["base_apy"]
    
    # Market cycle (90-day sine wave)
    market_cycle = math.sin(day * 2 * math.pi / 90) * 0.5
    
    # Weekly volatility
    weekly_volatility = math.sin(day * 2 * math.pi / 7) * base_volatility
    
    # Random daily variance
    daily_variance = random.gauss(0, protocol_config["variance"] * 0.3)
    
    # Chain multiplier
    apy = (base_apy + market_cycle + weekly_volatility + daily_variance) * chain_config["apy_multiplier"]
    
    # Ensure minimum 0.1% APY
    return max(0.1, apy)

def generate_tvl(protocol, day, volatility=0.1):
    """Generate realistic TVL with gradual changes"""
    base_tvl = PROTOCOLS[protocol]["base_tvl"]
    
    # Growth trend (slight upward trend over 90 days)
    growth_trend = day * 0.002  # 0.2% per day average growth
    
    # Market volatility
    market_volatility = math.sin(day * 2 * math.pi / 30) * volatility
    
    # Random daily change
    daily_change = random.gauss(0, volatility * 0.5)
    
    tvl = base_tvl * (1 + growth_trend + market_volatility + daily_change)
    
    return max(1_000_000, tvl)  # Minimum 1M TVL

def generate_gas_cost(chain, day):
    """Generate realistic gas costs with network congestion patterns"""
    chain_config = CHAINS[chain]
    
    # Base gas cost
    base_gas = chain_config["gas_base"]
    
    # Weekly pattern (higher on weekdays)
    day_of_week = day % 7
    weekday_multiplier = 1.2 if day_of_week < 5 else 0.8
    
    # Random variance
    variance = random.gauss(0, chain_config["gas_variance"] * 0.3)
    
    gas_cost = (base_gas + variance) * weekday_multiplier
    
    return max(0.1, gas_cost)

def generate_bridge_fee(chain, day):
    """Generate bridge fees with some variance"""
    base_fee = CHAINS[chain]["bridge_fee"]
    
    if base_fee == 0:  # Ethereum native
        return 0
    
    # Add some variance (±20%)
    variance = random.gauss(0, base_fee * 0.2)
    
    return max(1.0, base_fee + variance)

def generate_demo_data(filename="demo_yield_data_90_days.csv", days=DAYS):
    """Generate complete demo dataset"""
    
    print(f"Generating {days} days of yield optimization demo data...")
    
    with open(filename, 'w', newline='') as csvfile:
        fieldnames = ['date', 'protocol', 'chain', 'apy', 'stablecoin', 'tvl', 'gasCost', 'bridgeFee']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        
        for day in range(days):
            current_date = START_DATE + timedelta(days=day)
            date_str = current_date.strftime('%Y-%m-%d')
            
            for protocol in PROTOCOLS:
                for chain in CHAINS:
                    # Not all protocols available on all chains (realistic)
                    if protocol == "Pendle" and chain in ["Polygon"]:
                        continue
                    if protocol == "Convex" and chain != "Ethereum":
                        continue
                    
                    for stablecoin in STABLECOINS:
                        # Skip some combinations for realism
                        if random.random() < 0.1:  # 10% chance to skip
                            continue
                            
                        row = {
                            'date': date_str,
                            'protocol': protocol,
                            'chain': chain,
                            'apy': round(generate_apy(protocol, chain, day), 2),
                            'stablecoin': stablecoin,
                            'tvl': round(generate_tvl(protocol, day)),
                            'gasCost': round(generate_gas_cost(chain, day), 2),
                            'bridgeFee': round(generate_bridge_fee(chain, day), 2)
                        }
                        
                        writer.writerow(row)
    
    print(f"✅ Generated {filename} with realistic yield data")
    print(f"📊 Data includes:")
    print(f"   - {len(PROTOCOLS)} protocols: {', '.join(PROTOCOLS.keys())}")
    print(f"   - {len(CHAINS)} chains: {', '.join(CHAINS.keys())}")
    print(f"   - {len(STABLECOINS)} stablecoins: {', '.join(STABLECOINS)}")
    print(f"   - {days} days of historical data")
    print(f"\n🎯 Use this file in the DeFlow Yield Optimization Demo!")

def generate_custom_scenario(filename, scenario="conservative"):
    """Generate data for specific scenarios"""
    scenarios = {
        "conservative": {
            "protocols": ["Aave", "Compound", "Yearn"],
            "chains": ["Ethereum", "Arbitrum"],
            "apy_adjustment": 0.8,
            "volatility": 0.5
        },
        "aggressive": {
            "protocols": ["Pendle", "Curve", "Uniswap V3", "Balancer"],
            "chains": ["Arbitrum", "Polygon", "Optimism"],
            "apy_adjustment": 1.3,
            "volatility": 1.5
        },
        "balanced": {
            "protocols": list(PROTOCOLS.keys()),
            "chains": list(CHAINS.keys()),
            "apy_adjustment": 1.0,
            "volatility": 1.0
        }
    }
    
    if scenario not in scenarios:
        print(f"❌ Unknown scenario. Available: {list(scenarios.keys())}")
        return
    
    config = scenarios[scenario]
    print(f"Generating {scenario} scenario data...")
    
    # Temporarily modify global configs
    original_protocols = PROTOCOLS.copy()
    for protocol in PROTOCOLS:
        if protocol in config["protocols"]:
            PROTOCOLS[protocol]["base_apy"] *= config["apy_adjustment"]
            PROTOCOLS[protocol]["variance"] *= config["volatility"]
    
    generate_demo_data(filename)
    
    # Restore original configs
    PROTOCOLS.clear()
    PROTOCOLS.update(original_protocols)

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        if sys.argv[1] in ["conservative", "aggressive", "balanced"]:
            scenario = sys.argv[1]
            filename = f"demo_yield_data_{scenario}.csv"
            generate_custom_scenario(filename, scenario)
        else:
            print("Usage: python generate_demo_data.py [conservative|aggressive|balanced]")
            sys.exit(1)
    else:
        # Generate default comprehensive dataset
        generate_demo_data()
        
        # Also generate scenario files
        print("\n" + "="*50)
        print("Generating scenario-specific datasets...")
        for scenario in ["conservative", "aggressive", "balanced"]:
            generate_custom_scenario(f"demo_yield_data_{scenario}.csv", scenario)
    
    print("\n🎉 Demo data generation complete!")
    print("📁 Files generated:")
    print("   - demo_yield_data_90_days.csv (comprehensive)")
    print("   - demo_yield_data_conservative.csv")
    print("   - demo_yield_data_aggressive.csv") 
    print("   - demo_yield_data_balanced.csv")
    print("\n💡 Upload any of these files to the DeFlow demo at http://localhost:3000/demo/yield-optimization")