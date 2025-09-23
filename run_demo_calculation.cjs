#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * DeFlow Yield Optimization Demo Calculator
 * Runs the same calculations as the web demo but outputs results to files
 */

// Configuration - matches the web demo settings
const DEFAULT_SETTINGS = {
  initialAmount: 10000,
  riskTolerance: 'medium',
  minAPY: 0,
  maxGasCost: 100
};

// Chain allocation interface
class ChainAllocation {
  constructor(chain, amount, currentPosition = null) {
    this.chain = chain;
    this.amount = amount;
    this.currentPosition = currentPosition; // { protocol: string; chain: string } | null
  }
}

// Result interface
class OptimizationResult {
  constructor(day, date, selectedProtocol, selectedChain, apy, amount, gasCost, bridgeFee, netYield, cumulativeValue) {
    this.day = day;
    this.date = date;
    this.selectedProtocol = selectedProtocol;
    this.selectedChain = selectedChain;
    this.apy = apy;
    this.amount = amount;
    this.gasCost = gasCost;
    this.bridgeFee = bridgeFee;
    this.netYield = netYield;
    this.cumulativeValue = cumulativeValue;
  }
}

// Load CSV data
function loadCSVData(csvPath) {
  try {
    const csvContent = fs.readFileSync(csvPath, 'utf8');
    const lines = csvContent.trim().split('\n');
    const headers = lines[0].split(',');
    
    const data = [];
    for (let i = 1; i < lines.length; i++) {
      const values = lines[i].split(',');
      const row = {};
      headers.forEach((header, index) => {
        const value = values[index];
        if (['apy', 'tvl', 'gasCost', 'bridgeFee'].includes(header)) {
          row[header] = parseFloat(value) || 0;
        } else {
          row[header] = value;
        }
      });
      data.push(row);
    }
    
    console.log(`✅ Loaded ${data.length} data points from ${csvPath}`);
    return data;
  } catch (error) {
    console.error(`❌ Failed to load CSV data from ${csvPath}:`, error.message);
    return [];
  }
}

// L2-focused multi-chain allocation strategy
function allocateInitialCapital(totalAmount) {
  return [
    new ChainAllocation('Arbitrum', totalAmount * 0.5),   // 50% - Main base, good yields
    new ChainAllocation('Polygon', totalAmount * 0.25),   // 25% - Very low gas, high APY
    new ChainAllocation('Optimism', totalAmount * 0.15),  // 15% - Good balance 
    new ChainAllocation('Base', totalAmount * 0.1),       // 10% - Growing ecosystem
  ];
}

// Wave-riding optimization strategy with sticky thresholds
function findBestYieldForChain(dayData, chain, currentPosition, amount, settings) {
  // Filter by chain and user preferences
  const chainData = dayData.filter(d => d.chain === chain);
  const filtered = chainData.filter(d => 
    d.apy >= settings.minAPY &&
    d.gasCost <= settings.maxGasCost
  );

  if (filtered.length === 0) {
    return { option: chainData[0] || dayData[0], shouldMove: false, moveCost: 0 };
  }

  // Find the best option on this chain
  const bestOption = filtered.reduce((best, current) => 
    current.apy > best.apy ? current : best
  );

  // If we're not positioned anywhere on this chain, move to best option
  if (!currentPosition) {
    return { 
      option: bestOption, 
      shouldMove: true, 
      moveCost: bestOption.gasCost // No bridge fees within same chain
    };
  }

  // If we're already in the best position, stay put
  if (currentPosition.protocol === bestOption.protocol) {
    return { option: bestOption, shouldMove: false, moveCost: 0 };
  }

  // Calculate if it's worth moving within the same chain
  const currentOption = filtered.find(d => 
    d.protocol === currentPosition.protocol
  ) || bestOption;

  const apyDifference = bestOption.apy - currentOption.apy;
  const moveCost = bestOption.gasCost; // Only gas cost, no bridge fees
  
  // "Lazy/Wave-riding" strategy - only move when current position becomes weak
  // Stay in high APY positions until they fade, then find the next best opportunity
  
  const currentAPY = currentOption.apy;
  const bestAPY = bestOption.apy;
  
  // Calculate improvement needed based on current position strength
  let requiredImprovement;
  if (currentAPY >= 8.0) {
    // High APY position - very sticky, need big improvement to move
    requiredImprovement = 2.0;
  } else if (currentAPY >= 6.0) {
    // Good APY position - moderately sticky
    requiredImprovement = 1.5;
  } else if (currentAPY >= 4.0) {
    // Medium APY position - somewhat sticky
    requiredImprovement = 1.0;
  } else {
    // Low APY position - easy to move
    requiredImprovement = 0.5;
  }
  
  // Calculate if move is worth it (7-day payback + required improvement)
  const daily7DayBenefit = (amount * apyDifference / 365) / 100 * 7;
  
  if (daily7DayBenefit > moveCost && apyDifference > requiredImprovement) {
    return { option: bestOption, shouldMove: true, moveCost };
  } else {
    // Stay put and ride the current wave
    return { option: currentOption, shouldMove: false, moveCost: 0 };
  }
}

// Run the full optimization simulation
async function runOptimization(csvData, settings = DEFAULT_SETTINGS) {
  console.log('\n🚀 Starting DeFlow Yield Optimization Simulation...');
  console.log(`💰 Initial Amount: $${settings.initialAmount.toLocaleString()}`);
  console.log(`⚙️ Risk Tolerance: ${settings.riskTolerance}`);
  
  const results = [];
  let totalGasCosts = 0;
  let totalBridgeFees = 0;
  
  // Allocate capital across chains
  const chainAllocations = allocateInitialCapital(settings.initialAmount);
  console.log('\n📊 Multi-Chain Allocation:');
  chainAllocations.forEach(allocation => {
    console.log(`   ${allocation.chain}: $${allocation.amount.toLocaleString()} (${(allocation.amount/settings.initialAmount*100).toFixed(0)}%)`);
  });
  
  // Group data by date
  const dataByDate = csvData.reduce((acc, item) => {
    if (!acc[item.date]) acc[item.date] = [];
    acc[item.date].push(item);
    return acc;
  }, {});

  const dates = Object.keys(dataByDate).sort();
  
  console.log(`\n📅 Optimizing over ${Math.min(dates.length, 365)} days...`);
  
  for (let i = 0; i < Math.min(dates.length, 365); i++) {
    const date = dates[i];
    const dayData = dataByDate[date];
    
    if (!dayData || dayData.length === 0) continue;

    let totalDailyYield = 0;
    let totalDailyCosts = 0;
    let weightedAPY = 0;
    let primaryProtocol = '';
    let primaryChain = '';
    let totalCurrentAmount = 0;

    // Optimize each chain allocation separately
    for (const allocation of chainAllocations) {
      const decision = findBestYieldForChain(dayData, allocation.chain, allocation.currentPosition, allocation.amount, settings);
      
      // Daily yield calculation for this chain allocation
      const chainDailyYield = (allocation.amount * decision.option.apy / 365 / 100);
      totalDailyYield += chainDailyYield;
      
      // Add daily yield to this chain's allocation
      allocation.amount += chainDailyYield;
      
      // Only pay costs if we actually move
      if (decision.shouldMove) {
        const chainCosts = decision.moveCost;
        totalDailyCosts += chainCosts;
        totalGasCosts += chainCosts;
        allocation.amount -= chainCosts;
        allocation.currentPosition = {
          protocol: decision.option.protocol,
          chain: decision.option.chain
        };
      }
      
      // Calculate weighted metrics for display
      const chainWeight = allocation.amount / settings.initialAmount;
      weightedAPY += decision.option.apy * chainWeight;
      
      // Use the largest allocation for primary display
      if (allocation.amount > totalCurrentAmount) {
        totalCurrentAmount = allocation.amount;
        primaryProtocol = decision.option.protocol;
        primaryChain = decision.option.chain;
      }
    }

    // Calculate total portfolio value
    const totalPortfolioValue = chainAllocations.reduce((sum, alloc) => sum + alloc.amount, 0);

    const result = new OptimizationResult(
      i + 1,
      date,
      primaryProtocol,
      primaryChain,
      weightedAPY,
      totalPortfolioValue,
      totalDailyCosts,
      0, // No bridge fees in same-chain strategy
      totalDailyYield,
      totalPortfolioValue
    );

    results.push(result);
    
    // Progress indicator
    if (i % 30 === 29 || i === Math.min(dates.length, 365) - 1) {
      process.stdout.write(`   Day ${i + 1}/${Math.min(dates.length, 365)} ✅\n`);
    }
  }

  console.log('\n✨ Optimization Complete!');
  return results;
}

// Calculate comparison strategies
function calculateComparisonStrategies(csvData, optimizationResults, settings) {
  const finalDay = optimizationResults.length;
  
  // Static strategy - best single pool with setup costs
  const staticStrategy = (() => {
    if (csvData.length === 0) {
      const fallbackAPY = 8.5;
      const setupCost = 15;
      const grossValue = settings.initialAmount * (1 + (fallbackAPY / 100 / 365) * finalDay);
      return Math.max(settings.initialAmount, grossValue - setupCost);
    }
    
    // Find best performing pool
    const poolPerformance = csvData.reduce((acc, row) => {
      const poolKey = `${row.protocol}-${row.chain}`;
      if (!acc[poolKey]) {
        acc[poolKey] = { totalAPY: 0, count: 0, avgAPY: 0, avgGasCost: 0, avgBridgeFee: 0 };
      }
      acc[poolKey].totalAPY += row.apy || 0;
      acc[poolKey].avgGasCost += row.gasCost || 0;
      acc[poolKey].avgBridgeFee += row.bridgeFee || 0;
      acc[poolKey].count += 1;
      acc[poolKey].avgAPY = acc[poolKey].totalAPY / acc[poolKey].count;
      acc[poolKey].avgGasCost = acc[poolKey].avgGasCost / acc[poolKey].count;
      acc[poolKey].avgBridgeFee = acc[poolKey].avgBridgeFee / acc[poolKey].count;
      return acc;
    }, {});
    
    const bestPoolKey = Object.keys(poolPerformance).reduce((a, b) => 
      poolPerformance[a].avgAPY > poolPerformance[b].avgAPY ? a : b
    );
    
    let currentValue = settings.initialAmount;
    
    // Subtract initial setup cost
    const setupCost = poolPerformance[bestPoolKey].avgGasCost + poolPerformance[bestPoolKey].avgBridgeFee;
    currentValue -= setupCost;
    
    // Apply daily compounding
    const poolData = csvData
      .filter(row => `${row.protocol}-${row.chain}` === bestPoolKey)
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
    
    for (let day = 1; day <= finalDay; day++) {
      const dayIndex = Math.min(day - 1, poolData.length - 1);
      if (poolData[dayIndex]) {
        const dailyReturn = poolData[dayIndex].apy / 100 / 365;
        currentValue *= (1 + dailyReturn);
      } else {
        const avgAPY = poolPerformance[bestPoolKey].avgAPY;
        const dailyReturn = avgAPY / 100 / 365;
        currentValue *= (1 + dailyReturn);
      }
    }
    
    const finalValue = Math.max(settings.initialAmount * 0.5, currentValue);
    return { value: finalValue, pool: bestPoolKey };
  })();
  
  // Manual strategy - bi-weekly rebalancing with realistic costs
  const manualStrategy = (() => {
    if (csvData.length === 0) {
      const manualAPY = 8.5 * 1.1;
      const manualRebalances = Math.floor(finalDay / 14);
      const avgRebalanceCost = 8;
      const manualTotalFees = manualRebalances * avgRebalanceCost;
      const manualGrossValue = settings.initialAmount * (1 + (manualAPY / 100 / 365) * finalDay);
      return Math.max(settings.initialAmount, manualGrossValue - manualTotalFees);
    }
    
    let currentValue = settings.initialAmount;
    let totalFees = 0;
    
    for (let day = 1; day <= finalDay; day++) {
      if (day % 14 === 0 && day > 1) {
        // Calculate average rebalancing cost
        const dayData = csvData.filter(row => {
          const rowDate = new Date(row.date);
          const targetDate = new Date(csvData[0]?.date);
          targetDate.setDate(targetDate.getDate() + day - 1);
          return rowDate.toDateString() === targetDate.toDateString();
        });
        
        if (dayData.length > 0) {
          const topOptions = dayData.sort((a, b) => b.apy - a.apy).slice(0, 3);
          const avgGasCost = topOptions.reduce((sum, row) => sum + row.gasCost, 0) / topOptions.length;
          const avgBridgeFee = topOptions.reduce((sum, row) => sum + row.bridgeFee, 0) / topOptions.length;
          totalFees += avgGasCost + avgBridgeFee;
        } else {
          totalFees += 8;
        }
      }
      
      // Apply best daily APY
      const dayData = csvData.filter(row => {
        const rowDate = new Date(row.date);
        const targetDate = new Date(csvData[0]?.date);
        targetDate.setDate(targetDate.getDate() + day - 1);
        return rowDate.toDateString() === targetDate.toDateString();
      });
      
      if (dayData.length > 0) {
        const bestDailyAPY = Math.max(...dayData.map(row => row.apy));
        const dailyReturn = bestDailyAPY / 100 / 365;
        currentValue *= (1 + dailyReturn);
      }
    }
    
    return Math.max(settings.initialAmount * 0.5, currentValue - totalFees);
  })();
  
  // Bank savings
  const bankAPY = 4.0;
  const bankValue = settings.initialAmount * (1 + (bankAPY / 100 / 365) * finalDay);
  
  return {
    static: staticStrategy,
    manual: manualStrategy,
    bank: bankValue
  };
}

// Calculate summary statistics
function calculateSummary(optimizationResults, comparisons, settings) {
  if (optimizationResults.length === 0) return null;

  const finalValue = optimizationResults[optimizationResults.length - 1].cumulativeValue;
  const totalReturn = finalValue - settings.initialAmount;
  const totalReturnPercent = (totalReturn / settings.initialAmount) * 100;
  const annualizedReturn = (totalReturnPercent / optimizationResults.length) * 365;
  
  const totalGasCosts = optimizationResults.reduce((sum, r) => sum + r.gasCost, 0);
  const totalBridgeFees = optimizationResults.reduce((sum, r) => sum + r.bridgeFee, 0);
  const avgAPY = optimizationResults.reduce((sum, r) => sum + r.apy, 0) / optimizationResults.length;

  const protocolDistribution = optimizationResults.reduce((acc, r) => {
    acc[r.selectedProtocol] = (acc[r.selectedProtocol] || 0) + 1;
    return acc;
  }, {});

  return {
    finalValue,
    totalReturn,
    totalReturnPercent,
    annualizedReturn,
    totalGasCosts,
    totalBridgeFees,
    avgAPY,
    protocolDistribution,
    comparisons: {
      vsStatic: finalValue - (comparisons.static.value || comparisons.static),
      vsManual: finalValue - comparisons.manual,
      vsBank: finalValue - comparisons.bank,
      staticPool: comparisons.static.pool || 'Unknown'
    }
  };
}

// Save results to files
function saveResults(optimizationResults, summary, outputDir) {
  // Ensure output directory exists
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Save detailed daily results
  const detailedResults = {
    metadata: {
      generatedAt: new Date().toISOString(),
      strategy: "DeFlow Wave-Riding Optimization",
      totalDays: optimizationResults.length,
      initialAmount: 10000
    },
    dailyResults: optimizationResults.map(result => ({
      day: result.day,
      date: result.date,
      protocol: result.selectedProtocol,
      chain: result.selectedChain,
      apy: result.apy.toFixed(2),
      portfolioValue: result.cumulativeValue.toFixed(2),
      dailyYield: result.netYield.toFixed(2),
      costs: result.gasCost.toFixed(2)
    })),
    summary
  };
  
  fs.writeFileSync(
    path.join(outputDir, 'optimization_results.json'),
    JSON.stringify(detailedResults, null, 2)
  );

  // Save CSV for easy analysis
  const csvContent = [
    'Day,Date,Protocol,Chain,APY,Portfolio Value,Daily Yield,Costs',
    ...optimizationResults.map(r => 
      `${r.day},${r.date},${r.selectedProtocol},${r.selectedChain},${r.apy.toFixed(2)},${r.cumulativeValue.toFixed(2)},${r.netYield.toFixed(2)},${r.gasCost.toFixed(2)}`
    )
  ].join('\n');
  
  fs.writeFileSync(path.join(outputDir, 'optimization_results.csv'), csvContent);

  // Save summary report
  const summaryReport = `
# DeFlow Yield Optimization Results

Generated: ${new Date().toISOString()}

## Strategy Performance

**Final Portfolio Value:** $${summary.finalValue.toLocaleString()}
**Total Return:** $${summary.totalReturn.toFixed(2)} (${summary.totalReturnPercent.toFixed(2)}%)
**Annualized Return:** ${summary.annualizedReturn.toFixed(2)}%
**Average APY:** ${summary.avgAPY.toFixed(2)}%

## Cost Analysis

**Total Gas Costs:** $${summary.totalGasCosts.toFixed(2)}
**Total Bridge Fees:** $${summary.totalBridgeFees.toFixed(2)}
**Total Transaction Costs:** $${(summary.totalGasCosts + summary.totalBridgeFees).toFixed(2)}

## Strategy Comparisons

**vs Best Single Pool (${summary.comparisons.staticPool}):** +$${summary.comparisons.vsStatic.toFixed(2)}
**vs Manual Management:** +$${summary.comparisons.vsManual.toFixed(2)}
**vs High-Yield Savings:** +$${summary.comparisons.vsBank.toFixed(2)}

## Protocol Distribution

${Object.entries(summary.protocolDistribution)
  .sort(([,a], [,b]) => b - a)
  .map(([protocol, days]) => `**${protocol}:** ${days} days (${(days/optimizationResults.length*100).toFixed(1)}%)`)
  .join('\n')}

## Wave-Riding Strategy Highlights

- **L2-Focused:** Avoided expensive ETH L1 gas costs
- **Sticky Thresholds:** Stayed in high-APY positions until they faded
- **Cost-Benefit Analysis:** Only moved when 7-day payback + improvement threshold met
- **Multi-Chain Allocation:** 50% Arbitrum, 25% Polygon, 15% Optimism, 10% Base
`;

  fs.writeFileSync(path.join(outputDir, 'summary_report.md'), summaryReport);

  console.log(`\n📁 Results saved to ${outputDir}/`);
  console.log('   📄 optimization_results.json - Detailed JSON data');
  console.log('   📊 optimization_results.csv - CSV for analysis');
  console.log('   📋 summary_report.md - Human-readable summary');
}

// Main execution
async function main() {
  try {
    console.log('🎯 DeFlow Yield Optimization Demo Calculator');
    console.log('============================================');
    
    // Load CSV data
    const csvPath = process.argv[2] || '/Users/zhang/Desktop/ICP/DeFlow/demo_yield_data_90_days.csv';
    const outputDir = process.argv[3] || '/Users/zhang/Desktop/ICP/DeFlow/demo_result';
    
    console.log(`📊 Loading data from: ${csvPath}`);
    console.log(`📁 Output directory: ${outputDir}`);
    
    const csvData = loadCSVData(csvPath);
    if (csvData.length === 0) {
      console.error('❌ No data loaded. Exiting.');
      process.exit(1);
    }
    
    // Run optimization
    const optimizationResults = await runOptimization(csvData, DEFAULT_SETTINGS);
    
    // Calculate comparisons
    const comparisons = calculateComparisonStrategies(csvData, optimizationResults, DEFAULT_SETTINGS);
    
    // Calculate summary
    const summary = calculateSummary(optimizationResults, comparisons, DEFAULT_SETTINGS);
    
    // Save results
    saveResults(optimizationResults, summary, outputDir);
    
    // Print quick summary
    console.log('\n🎉 FINAL RESULTS');
    console.log('================');
    console.log(`💰 Final Value: $${summary.finalValue.toLocaleString()}`);
    console.log(`📈 Total Return: $${summary.totalReturn.toFixed(2)} (${summary.totalReturnPercent.toFixed(2)}%)`);
    console.log(`📊 Annualized: ${summary.annualizedReturn.toFixed(2)}%`);
    console.log(`💸 Total Costs: $${(summary.totalGasCosts + summary.totalBridgeFees).toFixed(2)}`);
    console.log(`🥇 vs Best Single Pool: +$${summary.comparisons.vsStatic.toFixed(2)}`);
    console.log(`🤝 vs Manual Management: +$${summary.comparisons.vsManual.toFixed(2)}`);
    console.log(`🏦 vs Bank Savings: +$${summary.comparisons.vsBank.toFixed(2)}`);
    
  } catch (error) {
    console.error('❌ Error running calculation:', error.message);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

module.exports = {
  runOptimization,
  calculateComparisonStrategies,
  calculateSummary,
  loadCSVData
};