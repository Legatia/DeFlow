/**
 * 🔒 Rate Limiter Service for DeFlow DeFi Operations
 * Implements token bucket algorithm with per-user and per-operation limiting
 * Prevents DoS attacks and ensures fair resource usage
 */

use candid::Principal;
use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// RATE LIMITING ERROR TYPES
// =============================================================================

#[derive(Debug, Clone)]
pub enum RateLimitError {
    RateLimitExceeded,
    TooManyRequests,
    BurstLimitExceeded,
    DailyLimitExceeded,
    Blocked,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            RateLimitError::TooManyRequests => write!(f, "Too many requests"),
            RateLimitError::BurstLimitExceeded => write!(f, "Burst limit exceeded"),
            RateLimitError::DailyLimitExceeded => write!(f, "Daily limit exceeded"),
            RateLimitError::Blocked => write!(f, "User is temporarily blocked"),
        }
    }
}

pub type RateLimitResult<T> = Result<T, RateLimitError>;

// =============================================================================
// TOKEN BUCKET IMPLEMENTATION
// =============================================================================

#[derive(Debug, Clone)]
pub struct TokenBucket {
    // Configuration
    capacity: u32,           // Maximum tokens
    refill_rate: u32,        // Tokens per second
    burst_capacity: u32,     // Burst limit
    
    // Current state
    tokens: f64,             // Current token count
    last_refill: Instant,    // Last refill time
    
    // Daily limits
    daily_requests: u32,     // Requests today
    daily_limit: u32,        // Max requests per day
    day_start: Instant,      // Start of current day
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32, burst_capacity: u32, daily_limit: u32) -> Self {
        let now = Instant::now();
        Self {
            capacity,
            refill_rate,
            burst_capacity,
            tokens: capacity as f64,
            last_refill: now,
            daily_requests: 0,
            daily_limit,
            day_start: now,
        }
    }
    
    // SECURITY: Check if request is allowed and consume token
    pub fn try_consume(&mut self, tokens_required: u32) -> RateLimitResult<()> {
        self.refill_tokens();
        self.check_daily_limit()?;
        
        if self.tokens >= tokens_required as f64 {
            self.tokens -= tokens_required as f64;
            self.daily_requests += 1;
            
            // Check burst limit
            if tokens_required > self.burst_capacity {
                return Err(RateLimitError::BurstLimitExceeded);
            }
            
            Ok(())
        } else {
            Err(RateLimitError::RateLimitExceeded)
        }
    }
    
    // Refill tokens based on elapsed time
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        let tokens_to_add = elapsed * self.refill_rate as f64;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
        self.last_refill = now;
    }
    
    // Check daily request limit
    fn check_daily_limit(&mut self) -> RateLimitResult<()> {
        let now = Instant::now();
        
        // Reset daily counter if it's a new day (24 hours)
        if now.duration_since(self.day_start) >= Duration::from_secs(86400) {
            self.daily_requests = 0;
            self.day_start = now;
        }
        
        if self.daily_requests >= self.daily_limit {
            return Err(RateLimitError::DailyLimitExceeded);
        }
        
        Ok(())
    }
    
    // Get remaining tokens
    pub fn available_tokens(&mut self) -> f64 {
        self.refill_tokens();
        self.tokens
    }
    
    // Get time until next token is available
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds_needed = tokens_needed / self.refill_rate as f64;
            Duration::from_secs_f64(seconds_needed)
        }
    }
}

// =============================================================================
// RATE LIMITING CONFIGURATION
// =============================================================================

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    // Per-second limits
    pub requests_per_second: u32,
    pub burst_capacity: u32,
    
    // Daily limits
    pub daily_limit: u32,
    
    // Per-operation limits
    pub send_bitcoin_limit: u32,       // Sends per hour
    pub send_ethereum_limit: u32,      // Sends per hour
    pub arbitrage_limit: u32,          // Arbitrage ops per hour
    pub portfolio_queries: u32,        // Portfolio queries per minute
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // Conservative defaults for financial operations
            requests_per_second: 10,     // 10 requests/second
            burst_capacity: 20,          // Allow bursts up to 20
            daily_limit: 1000,           // 1000 requests/day
            send_bitcoin_limit: 10,      // 10 sends/hour
            send_ethereum_limit: 10,     // 10 sends/hour
            arbitrage_limit: 100,        // 100 arbitrage ops/hour
            portfolio_queries: 60,       // 60 queries/minute
        }
    }
}

// Premium user configuration
impl RateLimitConfig {
    pub fn premium() -> Self {
        Self {
            requests_per_second: 50,     // 5x higher
            burst_capacity: 100,         // 5x higher
            daily_limit: 10000,          // 10x higher
            send_bitcoin_limit: 50,      // 5x higher
            send_ethereum_limit: 50,     // 5x higher
            arbitrage_limit: 500,        // 5x higher
            portfolio_queries: 300,      // 5x higher
        }
    }
}

// =============================================================================
// RATE LIMITER SERVICE
// =============================================================================

pub struct RateLimiterService {
    // Per-user general rate limits
    user_buckets: HashMap<Principal, TokenBucket>,
    
    // Per-user per-operation limits
    operation_buckets: HashMap<(Principal, String), TokenBucket>,
    
    // Blocked users (temporary bans)
    blocked_users: HashMap<Principal, Instant>,
    
    // Configuration
    default_config: RateLimitConfig,
    premium_config: RateLimitConfig,
    premium_users: Vec<Principal>,
}

impl RateLimiterService {
    pub fn new() -> Self {
        Self {
            user_buckets: HashMap::new(),
            operation_buckets: HashMap::new(),
            blocked_users: HashMap::new(),
            default_config: RateLimitConfig::default(),
            premium_config: RateLimitConfig::premium(),
            premium_users: Vec::new(),
        }
    }
    
    // SECURITY: Check if user is allowed to perform general request
    pub fn check_user_rate_limit(&mut self, user: Principal) -> RateLimitResult<()> {
        // Check if user is blocked
        if let Some(block_time) = self.blocked_users.get(&user) {
            let now = Instant::now();
            let block_duration = Duration::from_secs(3600); // 1 hour block
            
            if now.duration_since(*block_time) < block_duration {
                return Err(RateLimitError::Blocked);
            } else {
                // Unblock user
                self.blocked_users.remove(&user);
            }
        }
        
        // Get or create user bucket
        let config = if self.is_premium_user(&user) {
            &self.premium_config
        } else {
            &self.default_config
        };
        
        let bucket = self.user_buckets.entry(user).or_insert_with(|| {
            TokenBucket::new(
                config.requests_per_second * 60, // 1 minute capacity
                config.requests_per_second,
                config.burst_capacity,
                config.daily_limit,
            )
        });
        
        match bucket.try_consume(1) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Increment violation count and potentially block user
                self.handle_rate_limit_violation(user);
                Err(e)
            }
        }
    }
    
    // SECURITY: Check operation-specific rate limits
    pub fn check_operation_rate_limit(
        &mut self, 
        user: Principal, 
        operation: &str
    ) -> RateLimitResult<()> {
        let config = if self.is_premium_user(&user) {
            &self.premium_config
        } else {
            &self.default_config
        };
        
        // Get operation-specific limits
        let (capacity, refill_rate, daily_limit) = match operation {
            "send_bitcoin" => (
                config.send_bitcoin_limit,
                config.send_bitcoin_limit / 60, // Per minute
                config.send_bitcoin_limit * 24, // Per day
            ),
            "send_ethereum" => (
                config.send_ethereum_limit,
                config.send_ethereum_limit / 60,
                config.send_ethereum_limit * 24,
            ),
            "arbitrage" => (
                config.arbitrage_limit,
                config.arbitrage_limit / 60,
                config.arbitrage_limit * 24,
            ),
            "portfolio_query" => (
                config.portfolio_queries,
                config.portfolio_queries / 60,
                config.portfolio_queries * 24 * 60, // High limit for queries
            ),
            _ => (10, 1, 100), // Default conservative limits
        };
        
        let key = (user, operation.to_string());
        let bucket = self.operation_buckets.entry(key).or_insert_with(|| {
            TokenBucket::new(capacity, refill_rate, capacity / 2, daily_limit)
        });
        
        bucket.try_consume(1)
    }
    
    // SECURITY: Combined rate limiting check
    pub fn check_combined_limits(
        &mut self,
        user: Principal,
        operation: &str,
    ) -> RateLimitResult<()> {
        // Check general rate limit first
        self.check_user_rate_limit(user)?;
        
        // Then check operation-specific limit
        self.check_operation_rate_limit(user, operation)?;
        
        Ok(())
    }
    
    // Handle rate limit violations
    fn handle_rate_limit_violation(&mut self, user: Principal) {
        // For now, just log. In production, implement escalating penalties
        
        // Could implement: temporary blocks, escalating penalties, etc.
    }
    
    // Check if user has premium status
    fn is_premium_user(&self, user: &Principal) -> bool {
        self.premium_users.contains(user)
    }
    
    // Add premium user
    pub fn add_premium_user(&mut self, user: Principal) {
        if !self.premium_users.contains(&user) {
            self.premium_users.push(user);
        }
    }
    
    // Remove premium user
    pub fn remove_premium_user(&mut self, user: Principal) {
        self.premium_users.retain(|u| *u != user);
    }
    
    // Get rate limit status for user
    pub fn get_rate_limit_status(&mut self, user: Principal) -> RateLimitStatus {
        let general_tokens = self.user_buckets
            .get_mut(&user)
            .map(|b| b.available_tokens())
            .unwrap_or(0.0);
        
        let is_blocked = self.blocked_users.contains_key(&user);
        let is_premium = self.is_premium_user(&user);
        
        RateLimitStatus {
            available_tokens: general_tokens as u32,
            is_blocked,
            is_premium,
            daily_requests_remaining: self.user_buckets
                .get(&user)
                .map(|b| {
                    let config = if is_premium { &self.premium_config } else { &self.default_config };
                    config.daily_limit.saturating_sub(b.daily_requests)
                })
                .unwrap_or(0),
        }
    }
    
    // Clean up old buckets and blocked users (call periodically)
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        
        // Remove old blocked users (after 24 hours)
        self.blocked_users.retain(|_, block_time| {
            now.duration_since(*block_time) < Duration::from_secs(86400)
        });
        
        // Could also clean up unused buckets after some time
    }
}

impl Default for RateLimiterService {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// RATE LIMIT STATUS
// =============================================================================

#[derive(Debug, Clone, candid::CandidType, serde::Deserialize)]
pub struct RateLimitStatus {
    pub available_tokens: u32,
    pub is_blocked: bool,
    pub is_premium: bool,
    pub daily_requests_remaining: u32,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 1, 5, 100);
        
        // Should allow initial requests
        assert!(bucket.try_consume(1).is_ok());
        assert!(bucket.try_consume(5).is_ok());
        
        // Should reject when out of tokens
        assert!(bucket.try_consume(10).is_err());
        
        // Should reject burst that exceeds capacity
        assert!(bucket.try_consume(6).is_err());
    }
    
    #[test]
    fn test_rate_limiter_service() {
        let mut service = RateLimiterService::new();
        let user = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        
        // Should allow initial requests
        assert!(service.check_user_rate_limit(user).is_ok());
        assert!(service.check_operation_rate_limit(user, "send_bitcoin").is_ok());
        
        // Test combined limits
        assert!(service.check_combined_limits(user, "portfolio_query").is_ok());
    }
}