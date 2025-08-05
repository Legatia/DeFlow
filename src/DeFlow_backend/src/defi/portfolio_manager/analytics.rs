// Portfolio Analytics Engine
// Advanced analytics, performance metrics, and insights

use super::*;
use crate::defi::yield_farming::ChainId;
use ic_cdk::api::time;

/// Advanced portfolio analytics engine
#[derive(Debug, Clone)]
pub struct PortfolioAnalyticsEngine {
    pub performance_cache: HashMap<String, PerformanceCache>,
    pub benchmark_data: HashMap<String, BenchmarkData>,
    pub attribution_models: HashMap<String, AttributionModel>,
    pub market_data: MarketDataService,
    pub last_update: u64,
}

impl PortfolioAnalyticsEngine {
    pub fn new() -> Self {
        Self {
            performance_cache: HashMap::new(),
            benchmark_data: Self::initialize_benchmarks(),
            attribution_models: Self::initialize_attribution_models(),
            market_data: MarketDataService::new(),
            last_update: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.last_update = self.get_current_time();
        self.market_data.initialize();
    }

    /// Generate comprehensive portfolio analytics
    pub fn generate_portfolio_analytics(&self, portfolio: &UserPortfolio) -> Result<PortfolioAnalytics, PortfolioError> {
        let current_time = self.get_current_time();
        
        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(portfolio)?;
        
        // Calculate allocation breakdown
        let allocation_breakdown = self.calculate_allocation_breakdown(portfolio)?;
        
        // Calculate distributions
        let chain_distribution = self.calculate_chain_distribution(portfolio);
        let protocol_distribution = self.calculate_protocol_distribution(portfolio);
        
        // Calculate yield summary
        let yield_summary = self.calculate_yield_summary(portfolio)?;
        
        // Calculate attribution analysis
        let attribution_analysis = self.calculate_attribution_analysis(portfolio)?;
        
        // Calculate efficiency metrics
        let efficiency_metrics = self.calculate_efficiency_metrics(portfolio)?;
        
        // Calculate trend analysis
        let trend_analysis = self.calculate_trend_analysis(portfolio)?;

        Ok(PortfolioAnalytics {
            performance_metrics,
            allocation_breakdown,
            chain_distribution,
            protocol_distribution,
            yield_summary,
            attribution_analysis,
            efficiency_metrics,
            trend_analysis,
            generated_at: current_time,
        })
    }

    /// Store analytics for historical tracking
    pub fn store_analytics(&mut self, user_id: &str, analytics: PortfolioAnalytics) {
        let cache_entry = PerformanceCache {
            user_id: user_id.to_string(),
            analytics: analytics.clone(),
            cached_at: self.get_current_time(),
        };
        
        self.performance_cache.insert(user_id.to_string(), cache_entry);
    }

    /// Calculate portfolio performance over time period
    pub fn calculate_performance(&self, portfolio: &UserPortfolio, period_days: u32) -> Result<PortfolioPerformance, PortfolioError> {
        let current_value = portfolio.calculate_total_value();
        let current_time = self.get_current_time();
        
        // Mock historical data calculation
        let initial_value = self.estimate_historical_value(portfolio, period_days)?;
        let total_return = current_value - initial_value;
        let total_return_pct = if initial_value > 0.0 {
            (total_return / initial_value) * 100.0
        } else {
            0.0
        };
        
        // Annualized return
        let years = period_days as f64 / 365.0;
        let annualized_return = if years > 0.0 && initial_value > 0.0 {
            ((current_value / initial_value).powf(1.0 / years) - 1.0) * 100.0
        } else {
            0.0
        };
        
        // Calculate volatility (mock implementation)
        let volatility = self.calculate_portfolio_volatility(portfolio)?;
        
        // Calculate Sharpe ratio
        let risk_free_rate = 2.0; // 2% risk-free rate
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - risk_free_rate) / volatility
        } else {
            0.0
        };
        
        // Calculate maximum drawdown
        let max_drawdown = self.calculate_max_drawdown(portfolio, period_days)?;
        
        // Calculate Sortino ratio
        let downside_deviation = volatility * 0.7; // Mock downside deviation
        let sortino_ratio = if downside_deviation > 0.0 {
            (annualized_return - risk_free_rate) / downside_deviation
        } else {
            0.0
        };
        
        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            annualized_return / max_drawdown
        } else {
            0.0
        };

        Ok(PortfolioPerformance {
            period_days,
            initial_value,
            current_value,
            total_return,
            total_return_percentage: total_return_pct,
            annualized_return,
            volatility,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_drawdown,
            best_performing_position: self.find_best_performing_position(portfolio)?,
            worst_performing_position: self.find_worst_performing_position(portfolio)?,
            benchmark_comparison: self.calculate_benchmark_comparison(portfolio, period_days)?,
            risk_adjusted_return: annualized_return / volatility.max(1.0),
            win_rate: self.calculate_win_rate(portfolio)?,
            calculated_at: current_time,
        })
    }

    /// Analyze individual position
    pub fn analyze_position(&self, position: &Position) -> Result<PositionAnalytics, PortfolioError> {
        let current_time = self.get_current_time();
        let days_held = position.days_since_creation(current_time);
        
        // Calculate position performance
        let total_return = position.calculate_pnl();
        let total_return_pct = position.calculate_pnl_percentage();
        
        // Annualized return
        let years = days_held as f64 / 365.0;
        let annualized_return = if years > 0.0 && total_return_pct != 0.0 {
            ((1.0 + total_return_pct / 100.0).powf(1.0 / years) - 1.0) * 100.0
        } else {
            0.0
        };
        
        // Risk metrics
        let volatility = self.estimate_position_volatility(position)?;
        let var_1d = self.calculate_position_var(position, 1)?;
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - 2.0) / volatility // 2% risk-free rate
        } else {
            0.0
        };
        
        // Efficiency metrics
        let yield_efficiency = self.calculate_yield_efficiency(position)?;
        let capital_efficiency = self.calculate_capital_efficiency(position)?;
        let gas_efficiency = self.calculate_gas_efficiency(position)?;
        
        // Contribution analysis
        let portfolio_contribution = self.calculate_portfolio_contribution(position)?;
        
        // Risk contribution
        let risk_contribution = self.calculate_risk_contribution(position)?;

        Ok(PositionAnalytics {
            position_id: position.id.clone(),
            chain: position.chain.clone(),
            protocol: position.protocol.clone(),
            position_type: position.position_type.clone(),
            performance_metrics: PositionPerformanceMetrics {
                total_return,
                total_return_percentage: total_return_pct,
                annualized_return,
                volatility,
                sharpe_ratio,
                var_1d,
                days_held,
            },
            efficiency_metrics: PositionEfficiencyMetrics {
                yield_efficiency,
                capital_efficiency,
                gas_efficiency,
            },
            risk_metrics: PositionRiskMetrics {
                risk_score: position.risk_score,
                volatility,
                var_1d,
                risk_contribution,
            },
            contribution_analysis: ContributionAnalysis {
                portfolio_contribution,
                risk_contribution,
            },
            recommendations: self.generate_position_recommendations(position)?,
            analyzed_at: current_time,
        })
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, portfolio: &UserPortfolio) -> Result<PerformanceMetrics, PortfolioError> {
        let current_value = portfolio.calculate_total_value();
        let total_invested = portfolio.positions.iter().map(|p| p.initial_investment_usd).sum::<f64>();
        
        let total_pnl = current_value - total_invested;
        let total_pnl_pct = if total_invested > 0.0 {
            (total_pnl / total_invested) * 100.0
        } else {
            0.0
        };
        
        // Calculate average APY
        let weighted_apy = if current_value > 0.0 {
            portfolio.positions.iter()
                .map(|p| (p.value_usd / current_value) * p.current_apy)
                .sum()
        } else {
            0.0
        };
        
        // Calculate total rewards earned
        let total_rewards = portfolio.positions.iter()
            .map(|p| p.pending_rewards_usd + p.total_compounded_usd)
            .sum();
        
        // Days since portfolio creation
        let days_active = (self.get_current_time() - portfolio.created_at) / (24 * 3600 * 1_000_000_000);

        Ok(PerformanceMetrics {
            total_value_usd: current_value,
            total_invested_usd: total_invested,
            total_pnl_usd: total_pnl,
            total_pnl_percentage: total_pnl_pct,
            weighted_apy,
            total_rewards_usd: total_rewards,
            active_positions: portfolio.positions.len(),
            days_active,
        })
    }

    /// Calculate allocation breakdown
    fn calculate_allocation_breakdown(&self, portfolio: &UserPortfolio) -> Result<AllocationBreakdown, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        
        if total_value == 0.0 {
            return Ok(AllocationBreakdown {
                by_risk_level: HashMap::new(),
                by_position_type: HashMap::new(),
                by_yield_strategy: HashMap::new(),
                largest_position_pct: 0.0,
                diversification_score: 0.0,
            });
        }

        // Risk level breakdown
        let mut by_risk_level = HashMap::new();
        let mut low_risk = 0.0;
        let mut medium_risk = 0.0;
        let mut high_risk = 0.0;

        for position in &portfolio.positions {
            let weight = (position.value_usd / total_value) * 100.0;
            match position.risk_score {
                1..=3 => low_risk += weight,
                4..=6 => medium_risk += weight,
                7..=10 => high_risk += weight,
                _ => medium_risk += weight,
            }
        }

        by_risk_level.insert("Low".to_string(), low_risk);
        by_risk_level.insert("Medium".to_string(), medium_risk);
        by_risk_level.insert("High".to_string(), high_risk);

        // Position type breakdown
        let mut by_position_type = HashMap::new();
        for position in &portfolio.positions {
            let type_key = match &position.position_type {
                PositionType::YieldFarming { .. } => "Yield Farming",
                PositionType::Lending { .. } => "Lending",
                PositionType::LiquidityProvision { .. } => "Liquidity Provision",
                PositionType::Staking { .. } => "Staking",
                PositionType::Arbitrage { .. } => "Arbitrage",
            };
            let weight = (position.value_usd / total_value) * 100.0;
            let current = by_position_type.get(type_key).unwrap_or(&0.0);
            by_position_type.insert(type_key.to_string(), current + weight);
        }

        // Yield strategy breakdown (simplified)
        let mut by_yield_strategy = HashMap::new();
        let conservative_yield = portfolio.positions.iter()
            .filter(|p| p.current_apy <= 5.0)
            .map(|p| (p.value_usd / total_value) * 100.0)
            .sum();
        let moderate_yield = portfolio.positions.iter()
            .filter(|p| p.current_apy > 5.0 && p.current_apy <= 15.0)
            .map(|p| (p.value_usd / total_value) * 100.0)
            .sum();
        let aggressive_yield = portfolio.positions.iter()
            .filter(|p| p.current_apy > 15.0)
            .map(|p| (p.value_usd / total_value) * 100.0)
            .sum();

        by_yield_strategy.insert("Conservative (<=5%)".to_string(), conservative_yield);
        by_yield_strategy.insert("Moderate (5-15%)".to_string(), moderate_yield);
        by_yield_strategy.insert("Aggressive (>15%)".to_string(), aggressive_yield);

        // Largest position percentage
        let largest_position_pct = portfolio.positions.iter()
            .map(|p| (p.value_usd / total_value) * 100.0)
            .fold(0.0, f64::max);

        // Diversification score (based on number of positions and distribution)
        let diversification_score = self.calculate_diversification_score(portfolio);

        Ok(AllocationBreakdown {
            by_risk_level,
            by_position_type,
            by_yield_strategy,
            largest_position_pct,
            diversification_score,
        })
    }

    /// Calculate chain distribution
    fn calculate_chain_distribution(&self, portfolio: &UserPortfolio) -> HashMap<ChainId, f64> {
        let mut distribution = HashMap::new();
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return distribution;
        }

        for position in &portfolio.positions {
            let weight = (position.value_usd / total_value) * 100.0;
            let current = distribution.get(&position.chain).unwrap_or(&0.0);
            distribution.insert(position.chain.clone(), current + weight);
        }

        distribution
    }

    /// Calculate protocol distribution
    fn calculate_protocol_distribution(&self, portfolio: &UserPortfolio) -> HashMap<DeFiProtocol, f64> {
        let mut distribution = HashMap::new();
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return distribution;
        }

        for position in &portfolio.positions {
            let weight = (position.value_usd / total_value) * 100.0;
            let current = distribution.get(&position.protocol).unwrap_or(&0.0);
            distribution.insert(position.protocol.clone(), current + weight);
        }

        distribution
    }

    /// Calculate yield summary
    fn calculate_yield_summary(&self, portfolio: &UserPortfolio) -> Result<YieldSummary, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        
        if total_value == 0.0 {
            return Ok(YieldSummary {
                weighted_apy: 0.0,
                total_rewards_pending: 0.0,
                total_compounded: 0.0,
                estimated_monthly_yield: 0.0,
                estimated_annual_yield: 0.0,
                auto_compound_positions: 0,
                manual_compound_positions: 0,
            });
        }

        let weighted_apy = portfolio.positions.iter()
            .map(|p| (p.value_usd / total_value) * p.current_apy)
            .sum();

        let total_rewards_pending = portfolio.positions.iter()
            .map(|p| p.pending_rewards_usd)
            .sum();

        let total_compounded = portfolio.positions.iter()
            .map(|p| p.total_compounded_usd)
            .sum();

        let estimated_annual_yield = total_value * (weighted_apy / 100.0);
        let estimated_monthly_yield = estimated_annual_yield / 12.0;

        let auto_compound_positions = portfolio.positions.iter()
            .filter(|p| matches!(
                p.position_type,
                PositionType::YieldFarming { auto_compound: true, .. } |
                PositionType::LiquidityProvision { auto_compound: true, .. }
            ))
            .count();

        let manual_compound_positions = portfolio.positions.len() - auto_compound_positions;

        Ok(YieldSummary {
            weighted_apy,
            total_rewards_pending,
            total_compounded,
            estimated_monthly_yield,
            estimated_annual_yield,
            auto_compound_positions,
            manual_compound_positions,
        })
    }

    /// Calculate attribution analysis
    fn calculate_attribution_analysis(&self, portfolio: &UserPortfolio) -> Result<AttributionAnalysis, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        let total_pnl = portfolio.positions.iter().map(|p| p.calculate_pnl()).sum::<f64>();
        
        if total_value == 0.0 || total_pnl == 0.0 {
            return Ok(AttributionAnalysis {
                chain_attribution: HashMap::new(),
                protocol_attribution: HashMap::new(),
                strategy_attribution: HashMap::new(),
                best_contributor: None,
                worst_contributor: None,
            });
        }

        // Chain attribution
        let mut chain_attribution = HashMap::new();
        for position in &portfolio.positions {
            let contribution = position.calculate_pnl();
            let current = chain_attribution.get(&position.chain).unwrap_or(&0.0);
            chain_attribution.insert(position.chain.clone(), current + contribution);
        }

        // Protocol attribution
        let mut protocol_attribution = HashMap::new();
        for position in &portfolio.positions {
            let contribution = position.calculate_pnl();
            let current = protocol_attribution.get(&position.protocol).unwrap_or(&0.0);
            protocol_attribution.insert(position.protocol.clone(), current + contribution);
        }

        // Strategy attribution (simplified)
        let mut strategy_attribution = HashMap::new();
        for position in &portfolio.positions {
            let strategy_key = match &position.position_type {
                PositionType::YieldFarming { .. } => "Yield Farming",
                PositionType::Lending { .. } => "Lending",
                PositionType::LiquidityProvision { .. } => "Liquidity Provision",
                PositionType::Staking { .. } => "Staking",
                PositionType::Arbitrage { .. } => "Arbitrage",
            };
            let contribution = position.calculate_pnl();
            let current = strategy_attribution.get(strategy_key).unwrap_or(&0.0);
            strategy_attribution.insert(strategy_key.to_string(), current + contribution);
        }

        // Find best and worst contributors
        let best_contributor = portfolio.positions.iter()
            .max_by(|a, b| a.calculate_pnl().partial_cmp(&b.calculate_pnl()).unwrap())
            .map(|p| ContributorInfo {
                id: p.id.clone(),
                contribution: p.calculate_pnl(),
                contribution_percentage: (p.calculate_pnl() / total_pnl) * 100.0,
            });

        let worst_contributor = portfolio.positions.iter()
            .min_by(|a, b| a.calculate_pnl().partial_cmp(&b.calculate_pnl()).unwrap())
            .map(|p| ContributorInfo {
                id: p.id.clone(),
                contribution: p.calculate_pnl(),
                contribution_percentage: (p.calculate_pnl() / total_pnl) * 100.0,
            });

        Ok(AttributionAnalysis {
            chain_attribution,
            protocol_attribution,
            strategy_attribution,
            best_contributor,
            worst_contributor,
        })
    }

    /// Calculate efficiency metrics
    pub fn calculate_efficiency_metrics(&self, portfolio: &UserPortfolio) -> Result<EfficiencyMetrics, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        let total_rewards = portfolio.positions.iter().map(|p| p.pending_rewards_usd).sum::<f64>();
        
        // Capital efficiency (rewards generated per dollar invested)
        let capital_efficiency = if total_value > 0.0 {
            (total_rewards / total_value) * 100.0
        } else {
            0.0
        };

        // Gas efficiency (estimated)
        let estimated_gas_costs = self.estimate_total_gas_costs(portfolio)?;
        let gas_efficiency = if estimated_gas_costs > 0.0 {
            total_rewards / estimated_gas_costs
        } else {
            0.0
        };

        // Time efficiency (rewards per day)
        let portfolio_age_days = (self.get_current_time() - portfolio.created_at) / (24 * 3600 * 1_000_000_000);
        let time_efficiency = if portfolio_age_days > 0 {
            total_rewards / portfolio_age_days.max(1) as f64
        } else {
            0.0
        };

        // Risk efficiency (Sharpe-like ratio)
        let weighted_apy = if total_value > 0.0 {
            portfolio.positions.iter()
                .map(|p| (p.value_usd / total_value) * p.current_apy)
                .sum()
        } else {
            0.0
        };
        
        let portfolio_risk = self.calculate_portfolio_risk_score(portfolio)?;
        let risk_efficiency = if portfolio_risk > 0.0 {
            weighted_apy / portfolio_risk
        } else {
            0.0
        };

        Ok(EfficiencyMetrics {
            capital_efficiency,
            gas_efficiency,
            time_efficiency,
            risk_efficiency,
            compound_frequency_score: self.calculate_compound_frequency_score(portfolio)?,
        })
    }

    /// Calculate trend analysis
    fn calculate_trend_analysis(&self, _portfolio: &UserPortfolio) -> Result<TrendAnalysis, PortfolioError> {
        // Mock trend analysis - in production would use historical data
        Ok(TrendAnalysis {
            value_trend_7d: 2.5,     // 2.5% increase over 7 days
            value_trend_30d: 8.3,    // 8.3% increase over 30 days
            apy_trend_7d: -0.2,      // Slight APY decrease
            apy_trend_30d: 1.1,      // Overall APY increase
            risk_trend_7d: -1.0,     // Risk decreasing
            risk_trend_30d: 0.5,     // Slight risk increase over month
            momentum_score: 0.7,     // Positive momentum
            volatility_trend: -0.3,  // Volatility decreasing
        })
    }

    /// Helper functions
    fn estimate_historical_value(&self, portfolio: &UserPortfolio, days_ago: u32) -> Result<f64, PortfolioError> {
        // Mock historical value calculation
        let current_value = portfolio.calculate_total_value();
        let volatility_factor = self.calculate_portfolio_volatility(portfolio)? / 100.0;
        let time_factor = (days_ago as f64 / 365.0).min(1.0);
        
        // Simulate historical performance with some volatility
        let performance_factor = 1.0 - (0.05 * time_factor) + (volatility_factor * 0.1);
        Ok(current_value * performance_factor)
    }

    fn calculate_portfolio_volatility(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        if total_value == 0.0 {
            return Ok(0.0);
        }

        let mut weighted_volatility = 0.0;
        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let position_volatility = self.estimate_position_volatility(position)?;
            weighted_volatility += weight * position_volatility;
        }

        Ok(weighted_volatility)
    }

    fn estimate_position_volatility(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Mock volatility based on chain and protocol
        let base_volatility = match position.chain {
            ChainId::Bitcoin => 25.0,
            ChainId::Ethereum => 30.0,
            ChainId::Arbitrum => 32.0,
            ChainId::Solana => 45.0,
            _ => 35.0,
        };

        let protocol_factor = match position.protocol {
            DeFiProtocol::Aave => 0.8,
            DeFiProtocol::Compound => 0.8,
            DeFiProtocol::Uniswap(_) => 1.2,
            _ => 1.0,
        };

        Ok(base_volatility * protocol_factor)
    }

    fn calculate_max_drawdown(&self, _portfolio: &UserPortfolio, _period_days: u32) -> Result<f64, PortfolioError> {
        // Mock maximum drawdown calculation
        Ok(12.5) // 12.5% max drawdown
    }

    fn find_best_performing_position(&self, portfolio: &UserPortfolio) -> Result<Option<String>, PortfolioError> {
        Ok(portfolio.positions.iter()
            .max_by(|a, b| a.calculate_pnl_percentage().partial_cmp(&b.calculate_pnl_percentage()).unwrap())
            .map(|p| p.id.clone()))
    }

    fn find_worst_performing_position(&self, portfolio: &UserPortfolio) -> Result<Option<String>, PortfolioError> {
        Ok(portfolio.positions.iter()
            .min_by(|a, b| a.calculate_pnl_percentage().partial_cmp(&b.calculate_pnl_percentage()).unwrap())
            .map(|p| p.id.clone()))
    }

    fn calculate_benchmark_comparison(&self, _portfolio: &UserPortfolio, _period_days: u32) -> Result<BenchmarkComparison, PortfolioError> {
        // Mock benchmark comparison
        Ok(BenchmarkComparison {
            benchmark_name: "DeFi Index".to_string(),
            portfolio_return: 8.5,
            benchmark_return: 6.2,
            alpha: 2.3,
            beta: 1.1,
            tracking_error: 4.5,
            information_ratio: 0.51,
        })
    }

    fn calculate_win_rate(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let profitable_positions = portfolio.positions.iter()
            .filter(|p| p.calculate_pnl() > 0.0)
            .count();
        
        let total_positions = portfolio.positions.len();
        
        if total_positions > 0 {
            Ok((profitable_positions as f64 / total_positions as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn calculate_position_var(&self, _position: &Position, _days: u32) -> Result<f64, PortfolioError> {
        // Mock VaR calculation for individual position
        Ok(150.0) // $150 VaR
    }

    fn calculate_yield_efficiency(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Yield efficiency = APY / Risk Score
        if position.risk_score > 0 {
            Ok(position.current_apy / position.risk_score as f64)
        } else {
            Ok(0.0)
        }
    }

    fn calculate_capital_efficiency(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Capital efficiency = Total rewards / Initial investment
        if position.initial_investment_usd > 0.0 {
            Ok((position.pending_rewards_usd + position.total_compounded_usd) / position.initial_investment_usd * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn calculate_gas_efficiency(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Mock gas efficiency calculation
        let estimated_gas_cost = match position.chain {
            ChainId::Ethereum => 50.0,
            ChainId::Arbitrum => 10.0,
            ChainId::Polygon => 5.0,
            ChainId::Solana => 2.0,
            _ => 20.0,
        };

        if estimated_gas_cost > 0.0 {
            Ok(position.pending_rewards_usd / estimated_gas_cost)
        } else {
            Ok(0.0)
        }
    }

    fn calculate_portfolio_contribution(&self, _position: &Position) -> Result<f64, PortfolioError> {
        // Mock portfolio contribution
        Ok(5.2) // 5.2% contribution to portfolio performance
    }

    fn calculate_risk_contribution(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Risk contribution based on position size and risk score
        Ok(position.risk_score as f64 * 0.1)
    }

    fn generate_position_recommendations(&self, position: &Position) -> Result<Vec<String>, PortfolioError> {
        let mut recommendations = Vec::new();

        // Check performance
        if position.calculate_pnl_percentage() < -10.0 {
            recommendations.push("Consider reducing position size or exiting if fundamentals have changed".to_string());
        }

        // Check APY
        if position.current_apy < 3.0 {
            recommendations.push("Low yield - consider exploring higher-yield alternatives".to_string());
        }

        // Check risk
        if position.risk_score > 8 {
            recommendations.push("High risk position - monitor closely and consider risk management strategies".to_string());
        }

        // Auto-compound recommendation
        if matches!(position.position_type, PositionType::YieldFarming { auto_compound: false, .. }) {
            recommendations.push("Enable auto-compounding to maximize yield efficiency".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Position performing well - continue monitoring".to_string());
        }

        Ok(recommendations)
    }

    fn calculate_diversification_score(&self, portfolio: &UserPortfolio) -> f64 {
        if portfolio.positions.is_empty() {
            return 0.0;
        }

        let num_positions = portfolio.positions.len() as f64;
        let num_chains = portfolio.positions.iter()
            .map(|p| &p.chain)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;
        let num_protocols = portfolio.positions.iter()
            .map(|p| &p.protocol)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64;

        // Diversification score based on distribution across positions, chains, and protocols
        let position_score = (num_positions / 20.0).min(1.0) * 40.0; // Max 40 points for positions
        let chain_score = (num_chains / 5.0).min(1.0) * 30.0;       // Max 30 points for chains
        let protocol_score = (num_protocols / 5.0).min(1.0) * 30.0; // Max 30 points for protocols

        position_score + chain_score + protocol_score
    }

    fn estimate_total_gas_costs(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut total_gas = 0.0;
        
        for position in &portfolio.positions {
            let gas_cost = match position.chain {
                ChainId::Ethereum => 30.0,
                ChainId::Arbitrum => 8.0,
                ChainId::Polygon => 3.0,
                ChainId::Solana => 1.0,
                _ => 15.0,
            };
            total_gas += gas_cost;
        }

        Ok(total_gas)
    }

    fn calculate_portfolio_risk_score(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        if total_value == 0.0 {
            return Ok(0.0);
        }

        let weighted_risk = portfolio.positions.iter()
            .map(|p| (p.value_usd / total_value) * p.risk_score as f64)
            .sum();

        Ok(weighted_risk)
    }

    fn calculate_compound_frequency_score(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let auto_compound_count = portfolio.positions.iter()
            .filter(|p| matches!(
                p.position_type,
                PositionType::YieldFarming { auto_compound: true, .. } |
                PositionType::LiquidityProvision { auto_compound: true, .. }
            ))
            .count();

        let total_positions = portfolio.positions.len();
        
        if total_positions > 0 {
            Ok((auto_compound_count as f64 / total_positions as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn initialize_benchmarks() -> HashMap<String, BenchmarkData> {
        let mut benchmarks = HashMap::new();
        
        benchmarks.insert("defi_index".to_string(), BenchmarkData {
            name: "DeFi Total Return Index".to_string(),
            returns_7d: 3.2,
            returns_30d: 8.1,
            returns_90d: 15.6,
            volatility: 28.5,
        });

        benchmarks.insert("eth_staking".to_string(), BenchmarkData {
            name: "Ethereum Staking".to_string(),
            returns_7d: 0.1,
            returns_30d: 0.4,
            returns_90d: 1.2,
            volatility: 12.0,
        });

        benchmarks
    }

    fn initialize_attribution_models() -> HashMap<String, AttributionModel> {
        let mut models = HashMap::new();
        
        models.insert("chain_protocol".to_string(), AttributionModel {
            name: "Chain-Protocol Attribution".to_string(),
            factors: vec!["chain".to_string(), "protocol".to_string(), "strategy".to_string()],
            weights: vec![0.4, 0.4, 0.2],
        });

        models
    }

    fn get_current_time(&self) -> u64 {
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            if self.last_update == 0 || self.last_update == 1234567890_u64 {
                1234567890_u64
            } else {
                time()
            }
        }
    }
}

/// Market data service for analytics
#[derive(Debug, Clone)]
pub struct MarketDataService {
    pub price_cache: HashMap<String, f64>,
    pub last_updated: u64,
}

impl MarketDataService {
    pub fn new() -> Self {
        Self {
            price_cache: HashMap::new(),
            last_updated: 0,
        }
    }

    pub fn initialize(&mut self) {
        #[cfg(test)]
        {
            self.last_updated = 1234567890_u64;
        }
        #[cfg(not(test))]
        {
            self.last_updated = time();
        }
    }
}

/// Analytics data structures
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioAnalytics {
    pub performance_metrics: PerformanceMetrics,
    pub allocation_breakdown: AllocationBreakdown,
    pub chain_distribution: HashMap<ChainId, f64>,
    pub protocol_distribution: HashMap<DeFiProtocol, f64>,
    pub yield_summary: YieldSummary,
    pub attribution_analysis: AttributionAnalysis,
    pub efficiency_metrics: EfficiencyMetrics,
    pub trend_analysis: TrendAnalysis,
    pub generated_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_value_usd: f64,
    pub total_invested_usd: f64,
    pub total_pnl_usd: f64,
    pub total_pnl_percentage: f64,
    pub weighted_apy: f64,
    pub total_rewards_usd: f64,
    pub active_positions: usize,
    pub days_active: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AllocationBreakdown {
    pub by_risk_level: HashMap<String, f64>,
    pub by_position_type: HashMap<String, f64>,
    pub by_yield_strategy: HashMap<String, f64>,
    pub largest_position_pct: f64,
    pub diversification_score: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct YieldSummary {
    pub weighted_apy: f64,
    pub total_rewards_pending: f64,
    pub total_compounded: f64,
    pub estimated_monthly_yield: f64,
    pub estimated_annual_yield: f64,
    pub auto_compound_positions: usize,
    pub manual_compound_positions: usize,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AttributionAnalysis {
    pub chain_attribution: HashMap<ChainId, f64>,
    pub protocol_attribution: HashMap<DeFiProtocol, f64>,
    pub strategy_attribution: HashMap<String, f64>,
    pub best_contributor: Option<ContributorInfo>,
    pub worst_contributor: Option<ContributorInfo>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ContributorInfo {
    pub id: String,
    pub contribution: f64,
    pub contribution_percentage: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub capital_efficiency: f64,
    pub gas_efficiency: f64,
    pub time_efficiency: f64,
    pub risk_efficiency: f64,
    pub compound_frequency_score: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub value_trend_7d: f64,
    pub value_trend_30d: f64,
    pub apy_trend_7d: f64,
    pub apy_trend_30d: f64,
    pub risk_trend_7d: f64,
    pub risk_trend_30d: f64,
    pub momentum_score: f64,
    pub volatility_trend: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    pub period_days: u32,
    pub initial_value: f64,
    pub current_value: f64,
    pub total_return: f64,
    pub total_return_percentage: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub best_performing_position: Option<String>,
    pub worst_performing_position: Option<String>,
    pub benchmark_comparison: BenchmarkComparison,
    pub risk_adjusted_return: f64,
    pub win_rate: f64,
    pub calculated_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark_name: String,
    pub portfolio_return: f64,
    pub benchmark_return: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionAnalytics {
    pub position_id: String,
    pub chain: ChainId,
    pub protocol: DeFiProtocol,
    pub position_type: PositionType,
    pub performance_metrics: PositionPerformanceMetrics,
    pub efficiency_metrics: PositionEfficiencyMetrics,
    pub risk_metrics: PositionRiskMetrics,
    pub contribution_analysis: ContributionAnalysis,
    pub recommendations: Vec<String>,
    pub analyzed_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionPerformanceMetrics {
    pub total_return: f64,
    pub total_return_percentage: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub var_1d: f64,
    pub days_held: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionEfficiencyMetrics {
    pub yield_efficiency: f64,
    pub capital_efficiency: f64,
    pub gas_efficiency: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionRiskMetrics {
    pub risk_score: u8,
    pub volatility: f64,
    pub var_1d: f64,
    pub risk_contribution: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ContributionAnalysis {
    pub portfolio_contribution: f64,
    pub risk_contribution: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceCache {
    pub user_id: String,
    pub analytics: PortfolioAnalytics,
    pub cached_at: u64,
}

#[derive(Debug, Clone)]
pub struct BenchmarkData {
    pub name: String,
    pub returns_7d: f64,
    pub returns_30d: f64,
    pub returns_90d: f64,
    pub volatility: f64,
}

#[derive(Debug, Clone)]
pub struct AttributionModel {
    pub name: String,
    pub factors: Vec<String>,
    pub weights: Vec<f64>,
}