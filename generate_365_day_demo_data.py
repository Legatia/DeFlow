#!/usr/bin/env python3
"""
DeFlow 365-Day Yield Optimization Demo Data Generator

This script generates realistic 365-day CSV data that showcases the wave-riding strategy.
Includes realistic market cycles, seasonal patterns, and protocol-specific events.
"""

import csv
import random
import math
from datetime import datetime, timedelta

# Configuration for full year simulation
START_DATE = datetime(2024, 1, 1)
DAYS = 365

# Enhanced Protocol configurations with realistic market behavior
PROTOCOLS = {
    "Aave": {
        "base_apy": 4.0, 
        "variance": 2.0, 
        "base_tvl": 250_000_000, 
        "risk_factor": 0.8,
        "seasonal_factor": 0.3,  # Lower volatility, stable protocol
        "trend": 0.05,  # Slight upward trend over year
        "events": [  # Protocol-specific events throughout the year
            {"day": 45, "impact": 1.5, "duration": 14, "description": "Aave V4 launch"},
            {"day": 180, "impact": 0.7, "duration": 7, "description": "Regulatory concerns"},
            {"day": 300, "impact": 1.3, "duration": 21, "description": "New collateral types added"}
        ]
    },
    "Compound": {
        "base_apy": 3.8, 
        "variance": 1.8, 
        "base_tvl": 180_000_000, 
        "risk_factor": 0.7,
        "seasonal_factor": 0.4,
        "trend": -0.1,  # Slight decline due to competition
        "events": [
            {"day": 90, "impact": 1.2, "duration": 10, "description": "Compound III expansion"},
            {"day": 210, "impact": 0.8, "duration": 5, "description": "Oracle issues"}
        ]
    },
    "Curve": {
        "base_apy": 6.5, 
        "variance": 3.5, 
        "base_tvl": 320_000_000, 
        "risk_factor": 1.2,
        "seasonal_factor": 0.8,  # High volatility due to trading fees
        "trend": 0.2,
        "events": [
            {"day": 60, "impact": 2.5, "duration": 21, "description": "High volatility trading period - 16% APY"},
            {"day": 150, "impact": 0.5, "duration": 14, "description": "Low volume period"},
            {"day": 270, "impact": 2.2, "duration": 18, "description": "DeFi summer revival - 14% APY"}
        ],
        "spike_events": [
            {"day": 105, "impact": 3.0, "duration": 3, "description": "Depeg arbitrage - 17% APY"},
            {"day": 180, "impact": 2.8, "duration": 4, "description": "Stablecoin volatility - 15% APY"},
            {"day": 320, "impact": 3.3, "duration": 2, "description": "Flash arbitrage - 18% APY"}
        ]
    },
    "Uniswap V3": {
        "base_apy": 5.5,   # Lower base for dramatic spikes
        "variance": 3.5, 
        "base_tvl": 95_000_000, 
        "risk_factor": 1.4,
        "seasonal_factor": 1.0,  # Very volatile due to concentrated liquidity
        "trend": 0.3,
        "events": [
            {"day": 30, "impact": 2.8, "duration": 5, "description": "New fee tiers - 15% APY spike"},
            {"day": 120, "impact": 4.0, "duration": 21, "description": "Meme coin season - 20%+ APY"},
            {"day": 200, "impact": 0.4, "duration": 20, "description": "Market downturn"},
            {"day": 330, "impact": 3.2, "duration": 12, "description": "Altcoin rally - 17% APY"}
        ],
        "spike_events": [  # Flash opportunities
            {"day": 75, "impact": 3.8, "duration": 2, "description": "MEV spike - 19% APY"},
            {"day": 145, "impact": 3.4, "duration": 4, "description": "Volume surge - 16% APY"},
            {"day": 290, "impact": 3.6, "duration": 3, "description": "Arbitrage opportunity - 18% APY"}
        ]
    },
    "Yearn": {
        "base_apy": 5.1, 
        "variance": 2.2, 
        "base_tvl": 120_000_000, 
        "risk_factor": 0.9,
        "seasonal_factor": 0.5,
        "trend": 0.15,
        "events": [
            {"day": 75, "impact": 1.4, "duration": 20, "description": "New vault strategies"},
            {"day": 225, "impact": 1.6, "duration": 35, "description": "Yield farming boom"}
        ]
    },
    "Pendle": {
        "base_apy": 8.0,   # Lower base to make spikes more dramatic
        "variance": 4.0,   # High variance to show opportunity
        "base_tvl": 85_000_000, 
        "risk_factor": 2.0,
        "seasonal_factor": 1.2,  # High volatility due to yield trading
        "trend": 0.3,  # Moderate growth throughout year
        "events": [
            {"day": 50, "impact": 4.5, "duration": 7, "description": "Major yield token listing - 25%+ APY spike"},
            {"day": 110, "impact": 0.3, "duration": 14, "description": "Yield token expiry - massive drop"},
            {"day": 160, "impact": 3.8, "duration": 12, "description": "New yield source - 20%+ APY"},
            {"day": 240, "impact": 0.4, "duration": 21, "description": "Market correction - low yields"},
            {"day": 310, "impact": 4.2, "duration": 14, "description": "Real yield narrative boom - 22%+ APY"}
        ],
        "real_yield_focus": True,  # Special handling for real yield
        "spike_events": [  # Additional short-term spikes
            {"day": 85, "impact": 3.5, "duration": 3, "description": "Flash 18% APY opportunity"},
            {"day": 195, "impact": 4.0, "duration": 5, "description": "Arbitrage opportunity 20% APY"},
            {"day": 275, "impact": 3.2, "duration": 4, "description": "Liquidation event 16% APY"}
        ]
    },
    "Convex": {
        "base_apy": 5.8, 
        "variance": 2.5, 
        "base_tvl": 150_000_000, 
        "risk_factor": 1.0,
        "seasonal_factor": 0.6,
        "trend": 0.1,
        "events": [
            {"day": 100, "impact": 1.3, "duration": 15, "description": "CVX token staking boost"},
            {"day": 280, "impact": 1.1, "duration": 12, "description": "New Curve pool incentives"}
        ]
    },
    "Balancer": {
        "base_apy": 6.8, 
        "variance": 3.0, 
        "base_tvl": 110_000_000, 
        "risk_factor": 1.1,
        "seasonal_factor": 0.7,
        "trend": 0.25,
        "events": [
            {"day": 135, "impact": 1.6, "duration": 25, "description": "Balancer V3 features"},
            {"day": 250, "impact": 1.4, "duration": 18, "description": "New pool types launch"}
        ]
    },
}

# Enhanced chain configurations with seasonal gas patterns
CHAINS = {
    "Ethereum": {
        "gas_base": 18, "gas_variance": 12, "bridge_fee": 0, "apy_multiplier": 0.9,
        "seasonal_gas": True  # Higher gas during busy periods
    },
    "Arbitrum": {
        "gas_base": 2.5, "gas_variance": 1.5, "bridge_fee": 6, "apy_multiplier": 1.12,
        "seasonal_gas": False
    },
    "Optimism": {
        "gas_base": 1.8, "gas_variance": 1.0, "bridge_fee": 5, "apy_multiplier": 1.08,
        "seasonal_gas": False
    },
    "Polygon": {
        "gas_base": 1.0, "gas_variance": 0.6, "bridge_fee": 4, "apy_multiplier": 1.18,
        "seasonal_gas": False
    },
    "Base": {
        "gas_base": 1.5, "gas_variance": 0.8, "bridge_fee": 5, "apy_multiplier": 1.10,
        "seasonal_gas": False
    },
}

STABLECOINS = ["USDC", "USDT", "DAI"]

def get_market_cycle_factor(day):
    """Generate realistic market cycles throughout the year"""
    # Main cycle: Bear -> Recovery -> Bull -> Correction (roughly quarterly)
    main_cycle = math.sin(day * 2 * math.pi / 365) * 0.3
    
    # Secondary cycles for volatility
    quarterly_cycle = math.sin(day * 2 * math.pi / 90) * 0.2
    monthly_cycle = math.sin(day * 2 * math.pi / 30) * 0.1
    
    # Seasonal effects (DeFi summer, year-end)
    seasonal = 0
    if 150 <= day <= 240:  # Summer period
        seasonal = 0.4
    elif 300 <= day <= 365:  # Year-end speculation
        seasonal = 0.3
    elif 1 <= day <= 60:  # New year correction
        seasonal = -0.2
    
    return main_cycle + quarterly_cycle + monthly_cycle + seasonal

def get_protocol_event_impact(protocol, day):
    """Calculate impact from protocol-specific events"""
    protocol_config = PROTOCOLS[protocol]
    impact = 1.0
    
    # Regular events
    if "events" in protocol_config:
        for event in protocol_config["events"]:
            if event["day"] <= day < event["day"] + event["duration"]:
                # Fade the impact over duration
                fade_factor = 1 - (day - event["day"]) / event["duration"]
                impact *= 1 + (event["impact"] - 1) * fade_factor
    
    # Spike events (shorter, more intense)
    if "spike_events" in protocol_config:
        for event in protocol_config["spike_events"]:
            if event["day"] <= day < event["day"] + event["duration"]:
                # Less fading for spike events - stay intense
                fade_factor = 1 - ((day - event["day"]) / event["duration"]) * 0.3
                impact *= 1 + (event["impact"] - 1) * fade_factor
    
    return impact

def generate_apy(protocol, chain, day):
    """Generate realistic APY with complex market dynamics"""
    protocol_config = PROTOCOLS[protocol]
    chain_config = CHAINS[chain]
    
    # Base APY with yearly trend
    base_apy = protocol_config["base_apy"]
    yearly_trend = protocol_config["trend"] * (day / 365)
    
    # Market cycle effects
    market_factor = get_market_cycle_factor(day)
    
    # Protocol-specific events
    event_impact = get_protocol_event_impact(protocol, day)
    
    # Seasonal volatility
    seasonal_volatility = protocol_config["seasonal_factor"] * market_factor
    
    # Random daily variance
    daily_variance = random.gauss(0, protocol_config["variance"] * 0.4)
    
    # Weekly patterns (lower yields on weekends)
    day_of_week = day % 7
    weekend_factor = 0.95 if day_of_week >= 5 else 1.0
    
    # Chain multiplier with some variance
    chain_multiplier = chain_config["apy_multiplier"] * random.gauss(1.0, 0.05)
    
    # Calculate final APY
    apy = (base_apy + yearly_trend + seasonal_volatility + daily_variance) * event_impact * weekend_factor * chain_multiplier
    
    # Special handling for Pendle real yield focus
    if protocol == "Pendle" and protocol_config.get("real_yield_focus"):
        # Add extra volatility for yield trading opportunities
        yield_trading_factor = math.sin(day * 2 * math.pi / 45) * 0.8  # ~6 week cycles
        apy *= (1 + yield_trading_factor)
    
    # Ensure minimum 0.5% APY
    return max(0.5, apy)

def generate_tvl(protocol, day):
    """Generate realistic TVL with growth and market cycles"""
    base_tvl = PROTOCOLS[protocol]["base_tvl"]
    
    # Overall DeFi growth trend
    growth_trend = day * 0.001  # 0.1% per day average growth
    
    # Market cycle impact on TVL
    market_cycle = get_market_cycle_factor(day) * 0.3
    
    # Protocol event impact
    event_impact = get_protocol_event_impact(protocol, day)
    
    # Random daily change
    daily_change = random.gauss(0, 0.05)
    
    # Weekend effect (lower activity)
    day_of_week = day % 7
    weekend_factor = 0.98 if day_of_week >= 5 else 1.0
    
    tvl = base_tvl * (1 + growth_trend + market_cycle + daily_change) * event_impact * weekend_factor
    
    return max(10_000_000, tvl)  # Minimum 10M TVL

def generate_gas_cost(chain, day):
    """Generate realistic gas costs with network congestion patterns"""
    chain_config = CHAINS[chain]
    
    base_gas = chain_config["gas_base"]
    
    # Seasonal patterns for Ethereum
    if chain == "Ethereum" and chain_config.get("seasonal_gas"):
        # Higher gas during busy periods
        market_factor = get_market_cycle_factor(day)
        congestion_multiplier = 1 + market_factor * 0.5
    else:
        congestion_multiplier = 1.0
    
    # Weekly pattern (higher on weekdays, especially Tuesday-Thursday)
    day_of_week = day % 7
    if day_of_week in [1, 2, 3]:  # Tue, Wed, Thu
        weekday_multiplier = 1.3
    elif day_of_week in [0, 6]:  # Mon, Sun
        weekday_multiplier = 0.8
    else:
        weekday_multiplier = 1.0
    
    # Random variance
    variance = random.gauss(0, chain_config["gas_variance"] * 0.4)
    
    gas_cost = (base_gas + variance) * weekday_multiplier * congestion_multiplier
    
    return max(0.5, gas_cost)

def generate_bridge_fee(chain, day):
    """Generate bridge fees with market volatility"""
    base_fee = CHAINS[chain]["bridge_fee"]
    
    if base_fee == 0:  # Ethereum native
        return 0
    
    # Market volatility affects bridge fees
    market_factor = get_market_cycle_factor(day)
    volatility_multiplier = 1 + market_factor * 0.2
    
    # Add some variance (±15%)
    variance = random.gauss(0, base_fee * 0.15)
    
    return max(2.0, (base_fee + variance) * volatility_multiplier)

def generate_365_day_data(filename="demo_yield_data_365_days.csv"):
    """Generate comprehensive 365-day dataset"""
    
    print(f"🚀 Generating 365 days of advanced yield optimization data...")
    print(f"📅 Date range: {START_DATE.strftime('%Y-%m-%d')} to {(START_DATE + timedelta(days=DAYS-1)).strftime('%Y-%m-%d')}")
    
    with open(filename, 'w', newline='') as csvfile:
        fieldnames = ['date', 'protocol', 'chain', 'apy', 'stablecoin', 'tvl', 'gasCost', 'bridgeFee']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        
        for day in range(DAYS):
            current_date = START_DATE + timedelta(days=day)
            date_str = current_date.strftime('%Y-%m-%d')
            
            # Progress indicator
            if day % 30 == 0 or day == DAYS - 1:
                print(f"   📊 Day {day + 1}/{DAYS} ({current_date.strftime('%b %d')}) - {(day+1)/DAYS*100:.0f}% complete")
            
            for protocol in PROTOCOLS:
                for chain in CHAINS:
                    # Protocol availability restrictions
                    if protocol == "Pendle" and chain in ["Polygon"]:
                        continue
                    if protocol == "Convex" and chain != "Ethereum":
                        continue
                    
                    for stablecoin in STABLECOINS:
                        # Skip some combinations for realism (10% chance)
                        if random.random() < 0.08:
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
    
    print(f"\n✅ Generated {filename} with realistic 365-day yield data")
    print(f"📊 Data includes:")
    print(f"   - {len(PROTOCOLS)} protocols with individual market events")
    print(f"   - {len(CHAINS)} chains with realistic gas patterns")
    print(f"   - {len(STABLECOINS)} stablecoins")
    print(f"   - 365 days of market cycles and seasonal effects")
    print(f"   - Enhanced Pendle real yield mechanics")
    print(f"\n🎯 This dataset will showcase the wave-riding strategy effectiveness!")
    print(f"🌊 Expect to see protocol switches during market events and yield waves")

def print_protocol_preview():
    """Print a preview of protocol characteristics"""
    print(f"\n📋 Protocol Characteristics Preview:")
    print(f"="*60)
    for protocol, config in PROTOCOLS.items():
        events_desc = f"{len(config.get('events', []))} events" if 'events' in config else "No events"
        real_yield = " (Real Yield Focus)" if config.get('real_yield_focus') else ""
        print(f"  🔸 {protocol}{real_yield}")
        print(f"     Base APY: {config['base_apy']}%, Variance: {config['variance']}%, {events_desc}")
        if 'events' in config:
            for event in config['events']:
                print(f"     - Day {event['day']}: {event['description']} ({event['impact']}x impact)")
        print()

if __name__ == "__main__":
    import sys
    
    print("🎯 DeFlow 365-Day Advanced Yield Data Generator")
    print("=" * 50)
    
    # Print protocol preview
    print_protocol_preview()
    
    # Generate main dataset
    generate_365_day_data()
    
    print(f"\n🎉 365-day data generation complete!")
    print(f"📁 File generated: demo_yield_data_365_days.csv")
    print(f"\n💡 Use this file to see the wave-riding strategy in action:")
    print(f"   ./run_demo.sh demo_yield_data_365_days.csv ./demo_result_365")