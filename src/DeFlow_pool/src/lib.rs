use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

mod types;
mod pool_manager;
mod business_model;
mod cross_chain;
mod analytics;

use types::*;
use pool_manager::PoolManager;
use business_model::DevTeamBusinessManager;
use cross_chain::CrossChainManager;
use analytics::PoolAnalytics;

// Memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type StableStorage<K, V> = StableBTreeMap<K, V, Memory>;

const POOL_STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const RESERVES_MEMORY_ID: MemoryId = MemoryId::new(1);
const BUSINESS_MODEL_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    static POOL_STATE: RefCell<PoolState> = RefCell::new(PoolState::default());
    static POOL_MANAGER: RefCell<PoolManager> = RefCell::new(PoolManager::new());
    static BUSINESS_MANAGER: RefCell<DevTeamBusinessManager> = RefCell::new(DevTeamBusinessManager::new());
    static CROSS_CHAIN_MANAGER: RefCell<CrossChainManager> = RefCell::new(CrossChainManager::new());
    static ANALYTICS: RefCell<PoolAnalytics> = RefCell::new(PoolAnalytics::new());
}

#[init]
fn init(owner: Option<Principal>) {
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // TEAM HIERARCHY: Set specified owner or deployer as project owner
        let owner_principal = owner.unwrap_or_else(|| ic_cdk::caller());
        pool_state.dev_team_business.team_hierarchy.owner_principal = owner_principal;
        
        // Business configuration
        pool_state.dev_team_business.minimum_distribution_threshold = 5000.0; // $5K minimum
        pool_state.dev_team_business.distribution_frequency = 2_629_800; // 30 days in seconds
        
        // Grant owner premium access automatically
        pool_state.dev_team_business.team_member_earnings.insert(owner_principal, 0.0);
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    // Store state in stable memory before upgrade
    // Implementation depends on stable structures setup
}

#[post_upgrade]
fn post_upgrade() {
    // Restore state from stable memory after upgrade
    // Implementation depends on stable structures setup
}

// =============================================================================
// TEAM HIERARCHY & AUTHORIZATION
// =============================================================================

fn get_team_member_role(caller: Principal) -> Option<TeamRole> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let hierarchy = &pool_state.dev_team_business.team_hierarchy;
        
        if caller == hierarchy.owner_principal {
            Some(TeamRole::Owner)
        } else if hierarchy.senior_managers.contains(&caller) {
            Some(TeamRole::SeniorManager)
        } else if hierarchy.operations_managers.contains(&caller) {
            Some(TeamRole::OperationsManager)
        } else if hierarchy.tech_managers.contains(&caller) {
            Some(TeamRole::TechManager)
        } else if hierarchy.developers.contains(&caller) {
            Some(TeamRole::Developer)
        } else {
            None
        }
    })
}

fn is_owner(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner))
}

fn is_manager_or_above(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner | TeamRole::SeniorManager | TeamRole::OperationsManager | TeamRole::TechManager))
}

fn is_dev_team_member(caller: Principal) -> bool {
    get_team_member_role(caller).is_some()
}

fn can_view_financial_data(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner | TeamRole::SeniorManager))
}

fn require_owner() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_owner(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Owner access required".to_string())
    }
}

fn require_manager_or_above() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_manager_or_above(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Manager access or above required".to_string())
    }
}

fn require_dev_team_member() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_dev_team_member(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Dev team membership required".to_string())
    }
}

// =============================================================================
// POOL STATE MANAGEMENT
// =============================================================================

#[query]
fn get_pool_state() -> Result<PoolState, String> {
    POOL_STATE.with(|state| {
        Ok(state.borrow().clone())
    })
}

#[query]
fn get_financial_overview() -> Result<FinancialOverview, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and senior managers can view full financial overview
    if !can_view_financial_data(caller) {
        return Err("Unauthorized: Financial data access restricted to Owner and Senior Managers".to_string());
    }
    
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().get_financial_overview(&state.borrow())
        })
    })
}

#[query]
fn get_bootstrap_progress() -> f64 {
    POOL_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            manager.borrow().get_bootstrap_progress(&state.borrow())
        })
    })
}

// =============================================================================
// FEE COLLECTION & BUSINESS MODEL
// =============================================================================

#[update]
fn deposit_fee(asset: Asset, amount: u64, tx_id: String, _user: Principal) -> Result<String, String> {
    POOL_MANAGER.with(|pool_manager| {
        BUSINESS_MANAGER.with(|business_manager| {
            POOL_STATE.with(|state| {
                let mut pool_state = state.borrow_mut();
                
                // Split fee: 70% to pool liquidity, 30% to dev team profit
                let pool_portion = (amount as f64 * 0.7) as u64;
                let profit_portion = amount as f64 * 0.3;
                
                // Add pool portion to reserves
                pool_manager.borrow_mut().add_to_reserves(&mut pool_state, asset.clone(), pool_portion)?;
                
                // Add profit portion to dev team business model
                business_manager.borrow_mut().add_transaction_fee_revenue(&mut pool_state, profit_portion)?;
                
                // Check for monthly profit distribution
                business_manager.borrow_mut().check_and_execute_profit_distribution(&mut pool_state)?;
                
                // Check if bootstrap thresholds are met
                pool_manager.borrow_mut().check_bootstrap_completion(&mut pool_state)?;
                
                Ok(format!("Fee deposited: {} pool, {} profit from tx {}", pool_portion, profit_portion, tx_id))
            })
        })
    })
}

#[update]
fn process_subscription_payment(user: Principal, amount: f64) -> Result<String, String> {
    BUSINESS_MANAGER.with(|business_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            business_manager.borrow_mut().process_subscription_payment(&mut pool_state, user, amount)?;
            Ok(format!("Subscription payment processed: ${} from {:?}", amount, user))
        })
    })
}

#[update]
fn withdraw_dev_earnings() -> Result<f64, String> {
    let caller = ic_cdk::caller();
    
    BUSINESS_MANAGER.with(|business_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            business_manager.borrow_mut().withdraw_dev_earnings(&mut pool_state, caller)
        })
    })
}

#[query]  
fn get_dev_earnings(dev_principal: Principal) -> f64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.team_member_earnings
            .get(&dev_principal)
            .copied()
            .unwrap_or(0.0)
    })
}

// =============================================================================
// LIQUIDITY MANAGEMENT
// =============================================================================

#[update]
fn add_liquidity(chain_id: ChainId, asset: Asset, amount: u64) -> Result<String, String> {
    require_manager_or_above()?; // SECURITY: Only managers and above can add liquidity
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().add_liquidity(&mut pool_state, chain_id, asset, amount)?;
            Ok(format!("Liquidity added: {} {} on {:?}", amount, asset_to_string(&asset), chain_id))
        })
    })
}

#[update]
fn withdraw_for_execution(asset: Asset, amount: u64) -> Result<String, String> {
    require_manager_or_above()?; // SECURITY: Only managers and above can execute withdrawals
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().withdraw_for_execution(&mut pool_state, asset, amount)
        })
    })
}

#[query]
fn get_asset_balance(chain_id: ChainId, asset: Asset) -> u64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.reserves.get(&chain_id)
            .and_then(|assets| assets.get(&asset))
            .map(|reserve| reserve.total_amount)
            .unwrap_or(0)
    })
}

#[query]
fn get_total_liquidity_usd() -> Result<f64, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and senior managers can view liquidity amounts
    if !can_view_financial_data(caller) {
        return Err("Unauthorized: Liquidity data access restricted to Owner and Senior Managers".to_string());
    }
    
    POOL_STATE.with(|state| {
        Ok(state.borrow().total_liquidity_usd)
    })
}

// =============================================================================
// CROSS-CHAIN OPERATIONS
// =============================================================================

#[update]
async fn detect_arbitrage_opportunities() -> Result<Vec<ArbitrageOpportunity>, String> {
    CROSS_CHAIN_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            manager.borrow().detect_arbitrage_opportunities(&state.borrow())
        })
    })
}

#[update]
async fn execute_cross_chain_trade(
    source_chain: ChainId, 
    dest_chain: ChainId, 
    asset: Asset, 
    amount: u64
) -> Result<String, String> {
    require_manager_or_above()?; // SECURITY: Managers and above can execute cross-chain trades
    
    CROSS_CHAIN_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            manager.borrow_mut().execute_cross_chain_trade(&mut pool_state, source_chain, dest_chain, asset, amount)
        })
    })
}

// =============================================================================
// POOL CONFIGURATION
// =============================================================================

#[update]
fn set_bootstrap_targets(targets: Vec<(Asset, u64)>) -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can change bootstrap targets
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            
            // SECURITY: Rate limiting - minimum 1 hour between bootstrap changes
            let current_time = ic_cdk::api::time();
            let min_time_between_changes = 60 * 60 * 1_000_000_000; // 1 hour in nanoseconds
            
            if current_time - pool_state.dev_team_business.team_hierarchy.last_team_change < min_time_between_changes {
                return Err("SECURITY: Bootstrap changes rate limited. Wait 1 hour between changes.".to_string());
            }
            
            pool_manager.borrow_mut().set_bootstrap_targets(&mut pool_state, targets)?;
            pool_state.dev_team_business.team_hierarchy.last_team_change = current_time;
            Ok("Bootstrap targets updated".to_string())
        })
    })
}

#[update]
fn activate_pool() -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can activate the pool
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().activate_pool(&mut pool_state)
        })
    })
}

#[update]
fn emergency_pause(reason: String) -> Result<String, String> {
    require_manager_or_above()?; // SECURITY: Managers and above can emergency pause
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().emergency_pause(&mut pool_state, reason)
        })
    })
}

// =============================================================================
// TEAM HIERARCHY MANAGEMENT  
// =============================================================================

#[update]
fn add_team_member(principal: Principal, role: TeamRole) -> Result<String, String> {
    let _caller = require_owner()?; // SECURITY: Only owner can add team members directly
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // SECURITY: Rate limiting - minimum 1 hour between team changes
        let current_time = ic_cdk::api::time();
        let min_time_between_changes = 60 * 60 * 1_000_000_000; // 1 hour in nanoseconds
        
        if current_time - pool_state.dev_team_business.team_hierarchy.last_team_change < min_time_between_changes {
            return Err("SECURITY: Team changes rate limited. Wait 1 hour between changes.".to_string());
        }
        
        // Add to appropriate role list
        match role {
            TeamRole::Owner => return Err("Cannot add additional owners".to_string()),
            TeamRole::SeniorManager => {
                if !pool_state.dev_team_business.team_hierarchy.senior_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.senior_managers.push(principal);
                }
            },
            TeamRole::OperationsManager => {
                if !pool_state.dev_team_business.team_hierarchy.operations_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.operations_managers.push(principal);
                }
            },
            TeamRole::TechManager => {
                if !pool_state.dev_team_business.team_hierarchy.tech_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.tech_managers.push(principal);
                }
            },
            TeamRole::Developer => {
                if !pool_state.dev_team_business.team_hierarchy.developers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.developers.push(principal);
                }
            },
        }
        
        // Grant premium access and initialize earnings
        pool_state.dev_team_business.team_member_earnings.insert(principal, 0.0);
        pool_state.dev_team_business.team_hierarchy.last_team_change = current_time;
        
        Ok(format!("Team member added successfully as {:?}", role))
    })
}

#[update] 
fn remove_team_member(principal: Principal) -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can remove team members
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        if principal == hierarchy.owner_principal {
            return Err("Cannot remove the owner".to_string());
        }
        
        // Remove from all role lists
        hierarchy.senior_managers.retain(|&x| x != principal);
        hierarchy.operations_managers.retain(|&x| x != principal);
        hierarchy.tech_managers.retain(|&x| x != principal);
        hierarchy.developers.retain(|&x| x != principal);
        
        // Keep their earnings but mark as removed
        hierarchy.last_team_change = ic_cdk::api::time();
        
        Ok("Team member removed successfully".to_string())
    })
}

#[update]
fn request_team_change(principal: Principal, new_role: TeamRole) -> Result<u64, String> {
    let caller = require_manager_or_above()?; // Managers can request changes
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        let request = TeamChangeRequest {
            request_type: TeamChangeType::AddMember,
            requester: caller,
            target_principal: principal,
            new_role: new_role.clone(),
            requires_owner_approval: !matches!(new_role, TeamRole::Developer), // Only dev additions need owner approval
            timestamp: ic_cdk::api::time(),
            approved: false,
            request_id: hierarchy.next_request_id,
        };
        
        let request_id = hierarchy.next_request_id;
        hierarchy.next_request_id += 1;
        hierarchy.pending_approvals.push(request);
        
        Ok(request_id)
    })
}

#[update]
fn approve_team_change(request_id: u64) -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can approve team changes
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        if let Some(pos) = hierarchy.pending_approvals.iter().position(|req| req.request_id == request_id) {
            let mut request = hierarchy.pending_approvals.remove(pos);
            request.approved = true;
            
            // Execute the approved change
            match request.request_type {
                TeamChangeType::AddMember => {
                    match request.new_role {
                        TeamRole::Owner => return Err("Cannot add additional owners".to_string()),
                        TeamRole::SeniorManager => hierarchy.senior_managers.push(request.target_principal),
                        TeamRole::OperationsManager => hierarchy.operations_managers.push(request.target_principal),
                        TeamRole::TechManager => hierarchy.tech_managers.push(request.target_principal),
                        TeamRole::Developer => hierarchy.developers.push(request.target_principal),
                    }
                    
                    // Grant premium access
                    pool_state.dev_team_business.team_member_earnings.insert(request.target_principal, 0.0);
                },
                _ => {} // Handle other change types as needed
            }
            
            Ok(format!("Team change approved and executed for request {}", request_id))
        } else {
            Err("Request not found".to_string())
        }
    })
}

#[query]
fn get_team_hierarchy() -> Result<TeamHierarchy, String> {
    require_dev_team_member()?; // SECURITY: Only team members can view hierarchy
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        Ok(pool_state.dev_team_business.team_hierarchy.clone())
    })
}

#[query]
fn get_my_role() -> Option<TeamRole> {
    let caller = ic_cdk::caller();
    get_team_member_role(caller)
}

#[query]
fn get_my_earnings() -> f64 {
    let caller = ic_cdk::caller();
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.team_member_earnings
            .get(&caller)
            .copied()
            .unwrap_or(0.0)
    })
}

// =============================================================================
// PREMIUM TIER ACCESS FOR DEV TEAM
// =============================================================================

#[query]
fn get_user_fee_rate(user: Principal) -> f64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Check if user is dev team member (gets premium+ access)
        if pool_state.dev_team_business.team_member_earnings.contains_key(&user) {
            0.001 // 0.1% - Premium+ tier for all dev team members
        } else {
            // For non-team members, would check their subscription tier
            // Default to free tier for now
            0.0085 // 0.85% - Free tier 
        }
    })
}

#[query]
fn get_user_tier_info(user: Principal) -> String {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        if let Some(role) = get_team_member_role(user) {
            format!("DEV TEAM - {:?} (Premium+ 0.1% fees, Unlimited volume)", role)
        } else {
            "Free Tier (0.85% fees, Unlimited volume)".to_string()
        }
    })
}

#[query]
fn is_premium_user(user: Principal) -> bool {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        // Dev team members automatically get premium access
        pool_state.dev_team_business.team_member_earnings.contains_key(&user)
    })
}

// =============================================================================
// ANALYTICS
// =============================================================================

#[query]
fn get_pool_analytics() -> String {
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().generate_analytics_report(&state.borrow())
        })
    })
}

#[query]
fn get_chain_distribution() -> Vec<(ChainId, f64)> {
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().get_chain_distribution(&state.borrow())
        })
    })
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn asset_to_string(asset: &Asset) -> String {
    match asset {
        Asset::BTC => "BTC".to_string(),
        Asset::ETH => "ETH".to_string(),
        Asset::USDC => "USDC".to_string(),
        Asset::USDT => "USDT".to_string(),
        Asset::DAI => "DAI".to_string(),
        Asset::SOL => "SOL".to_string(),
        Asset::MATIC => "MATIC".to_string(),
        Asset::AVAX => "AVAX".to_string(),
    }
}

// Export Candid interface
ic_cdk::export_candid!();