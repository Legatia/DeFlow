// DeFlow Pre-calculated Demo Data Loader
// Use this to instantly load demo results in the frontend

const DEMO_SCENARIOS = {
  "365_day_realistic": {
    name: "365-Day Realistic Wave-Riding",
    description: "Full year simulation with realistic market cycles and APY spikes",
    finalValue: 11270.175,
    totalReturn: 1270.18,
    returnPercent: 12.70,
    annualizedReturn: 12.70,
    totalCosts: 349.49,
    avgAPY: 16.20,
    protocolDistribution: {
      "Pendle": 159,
      "Curve": 80, 
      "Balancer": 73,
      "Uniswap V3": 46,
      "Yearn": 7
    },
    highlights: [
      "Captured 40% APY spikes during meme season",
      "Stayed in Pendle during high real yield periods",
      "Smart switching reduced from daily to strategic moves",
      "L2-focused approach minimized gas costs"
    ]
  },
  
  "90_day_conservative": {
    name: "90-Day Conservative Strategy", 
    description: "Lower risk approach with stable protocols",
    finalValue: 10251.998,
    totalReturn: 251.998,
    returnPercent: 2.52,
    annualizedReturn: 10.21,
    totalCosts: 45.23,
    avgAPY: 8.5,
    protocolDistribution: {
      "Aave": 35,
      "Compound": 28,
      "Yearn": 22,
      "Pendle": 5
    },
    highlights: [
      "Focus on blue-chip protocols",
      "Lower volatility and risk",
      "Consistent returns over time",
      "Minimal protocol switching"
    ]
  },

  "90_day_aggressive": {
    name: "90-Day Aggressive Strategy",
    description: "High-yield hunting with frequent optimization", 
    finalValue: 10330.504,
    totalReturn: 330.504,
    returnPercent: 3.31,
    annualizedReturn: 13.42,
    totalCosts: 89.12,
    avgAPY: 18.7,
    protocolDistribution: {
      "Pendle": 32,
      "Uniswap V3": 28,
      "Curve": 18,
      "Balancer": 12
    },
    highlights: [
      "Aggressive APY spike hunting",
      "Higher transaction costs but worth it",
      "Captures short-term opportunities",
      "Maximum yield optimization"
    ]
  }
};

// Function to load demo data into the frontend
function loadDemoScenario(scenarioKey) {
  const scenario = DEMO_SCENARIOS[scenarioKey];
  if (!scenario) {
    console.error('Demo scenario not found:', scenarioKey);
    return null;
  }

  console.log(`🎯 Loading Demo: ${scenario.name}`);
  console.log(`📊 ${scenario.description}`);
  console.log(`💰 Final Value: $${scenario.finalValue.toLocaleString()}`);
  console.log(`📈 Return: ${scenario.returnPercent}% (${scenario.annualizedReturn}% annualized)`);
  console.log(`💸 Total Costs: $${scenario.totalCosts}`);
  
  return scenario;
}

// Quick demo launcher for web interface
function quickWebDemo() {
  console.log('🎯 DeFlow Quick Web Demo');
  console.log('========================');
  console.log('Available scenarios:');
  
  Object.keys(DEMO_SCENARIOS).forEach((key, index) => {
    const scenario = DEMO_SCENARIOS[key];
    console.log(`${index + 1}. ${scenario.name} - ${scenario.returnPercent}% return`);
  });
  
  return DEMO_SCENARIOS;
}

// Export for use in React components
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    DEMO_SCENARIOS,
    loadDemoScenario,
    quickWebDemo
  };
}

// Browser global
if (typeof window !== 'undefined') {
  window.DeFlowDemo = {
    scenarios: DEMO_SCENARIOS,
    load: loadDemoScenario,
    quick: quickWebDemo
  };
}