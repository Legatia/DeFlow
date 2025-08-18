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
        
        // SECURITY: Validate and set owner principal
        let caller = ic_cdk::caller();
        let owner_principal = owner.unwrap_or(caller);
        
        // SECURITY: Prevent anonymous principal as owner
        if owner_principal == Principal::anonymous() {
            ic_cdk::trap("SECURITY: Cannot initialize with anonymous principal as owner");
        }
        
        // SECURITY: Log initialization for audit
        ic_cdk::println!("AUDIT: Canister initialized - Owner: {}, Caller: {}", 
                         owner_principal.to_text(), caller.to_text());
        
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
    // SECURITY: Store critical state in stable memory before upgrade
    ic_cdk::println!("SECURITY: Starting canister upgrade - preserving state");
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Store the pool state in stable memory
        MEMORY_MANAGER.with(|m| {
            let memory = m.borrow().get(POOL_STATE_MEMORY_ID);
            let mut stable_storage: StableStorage<u64, PoolState> = 
                StableBTreeMap::init(memory);
            
            match stable_storage.insert(0, pool_state.clone()) {
                Some(_) => ic_cdk::println!("SECURITY: Pool state updated in stable memory"),
                None => ic_cdk::println!("SECURITY: Pool state stored in stable memory"),
            }
        });
        
        // Log critical metrics before upgrade
        ic_cdk::println!("AUDIT: Pre-upgrade - Total liquidity: ${}, Team members: {}, Treasury transactions: {}", 
                         pool_state.total_liquidity_usd,
                         pool_state.dev_team_business.team_member_earnings.len(),
                         pool_state.treasury_transactions.len());
    });
    
    ic_cdk::println!("SECURITY: Pre-upgrade state preservation completed");
}

#[post_upgrade]
fn post_upgrade() {
    // SECURITY: Restore critical state from stable memory after upgrade
    ic_cdk::println!("SECURITY: Starting post-upgrade state restoration");
    
    MEMORY_MANAGER.with(|m| {
        let memory = m.borrow().get(POOL_STATE_MEMORY_ID);
        let stable_storage: StableStorage<u64, PoolState> = 
            StableBTreeMap::init(memory);
        
        match stable_storage.get(&0) {
            Some(restored_state) => {
                POOL_STATE.with(|state| {
                    *state.borrow_mut() = restored_state;
                });
                
                ic_cdk::println!("SECURITY: Pool state successfully restored from stable memory");
                
                // Log critical metrics after upgrade
                POOL_STATE.with(|state| {
                    let pool_state = state.borrow();
                    ic_cdk::println!("AUDIT: Post-upgrade - Total liquidity: ${}, Team members: {}, Treasury transactions: {}", 
                                     pool_state.total_liquidity_usd,
                                     pool_state.dev_team_business.team_member_earnings.len(),
                                     pool_state.treasury_transactions.len());
                });
            }
            None => {
                ic_cdk::println!("WARNING: No saved state found in stable memory - using default state");
                // Initialize with default state
                POOL_STATE.with(|state| {
                    *state.borrow_mut() = PoolState::default();
                });
            }
        }
    });
    
    ic_cdk::println!("SECURITY: Post-upgrade state restoration completed");
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

fn is_authorized_fee_depositor(caller: Principal) -> bool {
    // SECURITY: Only managers and above can deposit fees, plus whitelisted services
    if is_manager_or_above(caller) {
        return true;
    }
    
    // Add whitelisted service principals here
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        // Check if caller is in authorized services list
        pool_state.treasury_config.withdrawal_approvers.contains(&caller)
    })
}

fn is_authorized_payment_processor(caller: Principal) -> bool {
    // SECURITY: Only managers and above can process payments, plus whitelisted services
    if is_manager_or_above(caller) {
        return true;
    }
    
    // Check if caller is in authorized payment processors list
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.treasury_config.withdrawal_approvers.contains(&caller)
    })
}

fn verify_financial_access_session(caller: Principal) -> bool {
    // SECURITY: Additional session verification for financial data access
    // In production, this would check for recent authentication, MFA, etc.
    
    let current_time = ic_cdk::api::time();
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Check if caller has had recent activity (within last hour)
        // This is a simplified check - production should implement proper session management
        let hierarchy = &pool_state.dev_team_business.team_hierarchy;
        
        // Owner and senior managers get longer session validity
        if caller == hierarchy.owner_principal {
            true // Owner always has access
        } else if hierarchy.senior_managers.contains(&caller) {
            // Senior managers get 4 hour sessions
            current_time - hierarchy.last_team_change < (4 * 60 * 60 * 1_000_000_000)
        } else {
            // Others need more recent verification
            current_time - hierarchy.last_team_change < (1 * 60 * 60 * 1_000_000_000)
        }
    })
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
        // AUDIT: Log unauthorized access attempts
        ic_cdk::println!("SECURITY: Unauthorized financial data access attempt by {}", caller.to_text());
        return Err("Unauthorized: Financial data access restricted to Owner and Senior Managers".to_string());
    }
    
    // SECURITY: Additional verification for financial data access
    if !verify_financial_access_session(caller) {
        ic_cdk::println!("SECURITY: Financial access session verification failed for {}", caller.to_text());
        return Err("Session verification required for financial data access".to_string());
    }
    
    // AUDIT: Log successful financial data access
    ic_cdk::println!("AUDIT: Financial overview accessed by {} at {}", caller.to_text(), ic_cdk::api::time());
    
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
fn deposit_fee(asset: Asset, amount: u64, tx_id: String, user: Principal) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only authorized services can deposit fees
    if !is_authorized_fee_depositor(caller) {
        return Err("Unauthorized: Only authorized services can deposit fees".to_string());
    }
    
    // SECURITY: Input validation
    if amount == 0 {
        return Err("Invalid amount: Must be greater than 0".to_string());
    }
    
    if tx_id.is_empty() || tx_id.len() > 100 {
        return Err("Invalid transaction ID: Must be 1-100 characters".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Fee deposit - Asset: {:?}, Amount: {}, TxID: {}, User: {}, Caller: {}", 
                     asset, amount, tx_id, user.to_text(), caller.to_text());
    
    POOL_MANAGER.with(|pool_manager| {
        BUSINESS_MANAGER.with(|business_manager| {
            POOL_STATE.with(|state| {
                let mut pool_state = state.borrow_mut();
                
                // Split fee: 70% to pool liquidity, 30% to treasury (unified dev wallet)
                let pool_portion = (amount as f64 * 0.7) as u64;
                let treasury_portion = amount as f64 * 0.3;
                
                // Add pool portion to reserves
                pool_manager.borrow_mut().add_to_reserves(&mut pool_state, asset.clone(), pool_portion)?;
                
                // Record treasury transaction (30% of transaction fee)
                let treasury_tx = TreasuryTransaction {
                    id: format!("fee_{}", tx_id),
                    transaction_type: TreasuryTransactionType::TransactionFeeRevenue,
                    chain: "icp".to_string(), // Transaction fees collected in ICP
                    asset: asset.to_string(),
                    amount: treasury_portion,
                    amount_usd: treasury_portion, // Assuming 1:1 for now, should use real price oracle
                    from_address: "pool".to_string(),
                    to_address: "treasury".to_string(),
                    tx_hash: Some(tx_id.clone()),
                    status: TransactionStatus::Confirmed,
                    timestamp: ic_cdk::api::time(),
                    initiated_by: ic_cdk::caller(),
                    notes: Some("30% of transaction fee automatically allocated to treasury".to_string()),
                };
                
                // Add to treasury transactions and update balances
                pool_state.treasury_transactions.push(treasury_tx);
                
                // Update treasury balance for this asset
                let asset_string = asset.to_string();
                if let Some(balance) = pool_state.treasury_balances.iter_mut()
                    .find(|b| b.chain == "icp" && b.asset == asset_string) {
                    balance.amount += treasury_portion;
                    balance.amount_usd += treasury_portion;
                    balance.last_updated = ic_cdk::api::time();
                } else {
                    // Create new treasury balance entry
                    pool_state.treasury_balances.push(TreasuryBalance {
                        chain: "icp".to_string(),
                        asset: asset_string.clone(),
                        amount: treasury_portion,
                        amount_usd: treasury_portion,
                        last_updated: ic_cdk::api::time(),
                        last_tx_hash: Some(tx_id.clone()),
                    });
                }
                
                // Also add profit portion to legacy dev team business model (for backward compatibility)
                business_manager.borrow_mut().add_transaction_fee_revenue(&mut pool_state, treasury_portion)?;
                
                // Check for monthly profit distribution
                business_manager.borrow_mut().check_and_execute_profit_distribution(&mut pool_state)?;
                
                // Check if bootstrap thresholds are met
                pool_manager.borrow_mut().check_bootstrap_completion(&mut pool_state)?;
                
                Ok(format!("Fee deposited: {} pool, {} treasury from tx {}", pool_portion, treasury_portion, tx_id))
            })
        })
    })
}

#[update]
fn process_subscription_payment(user: Principal, amount: f64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only authorized payment processors can process subscriptions
    if !is_authorized_payment_processor(caller) {
        return Err("Unauthorized: Only authorized payment processors allowed".to_string());
    }
    
    // SECURITY: Input validation
    if amount <= 0.0 || amount > 100000.0 {
        return Err("Invalid amount: Must be between 0 and $100,000".to_string());
    }
    
    if user == Principal::anonymous() {
        return Err("Invalid user: Cannot process payment for anonymous principal".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Subscription payment - User: {}, Amount: ${}, Caller: {}", 
                     user.to_text(), amount, caller.to_text());
    
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
// TREASURY MANAGEMENT APIS
// =============================================================================

#[update]
fn configure_payment_address(
    chain: String,
    asset: String,
    address: String,
    address_type: AddressType,
    max_balance_usd: Option<f64>
) -> Result<(), String> {
    require_owner()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        let payment_address = PaymentAddress {
            chain: chain.clone(),
            asset: asset.clone(),
            address: address.clone(),
            address_type,
            max_balance_usd,
            created_at: current_time,
            last_used: 0,
        };
        
        // Remove existing address for this chain/asset combination
        pool_state.payment_addresses.retain(|addr| 
            !(addr.chain == chain && addr.asset == asset)
        );
        
        // Add new address
        pool_state.payment_addresses.push(payment_address);
        
        // Update treasury config map for quick lookup
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.payment_addresses.insert(key, address);
        
        Ok(())
    })
}

#[query]
fn get_payment_address(chain: String, asset: String) -> Option<String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.payment_addresses.get(&key).cloned()
    })
}

#[query]
fn get_all_payment_addresses() -> Vec<PaymentAddress> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new(); // Only managers can see all addresses
    }
    
    POOL_STATE.with(|state| {
        state.borrow().payment_addresses.clone()
    })
}

#[update]
fn set_hot_wallet_limit(chain: String, asset: String, limit_usd: f64) -> Result<(), String> {
    require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.hot_wallet_limits.insert(key, limit_usd);
        Ok(())
    })
}

#[update]
fn record_subscription_payment(
    user_principal: Principal,
    chain: String,
    asset: String,
    amount: f64,
    amount_usd: f64,
    tx_hash: String,
    subscription_tier: String
) -> Result<(), String> {
    require_manager_or_above()?;
    
    // SECURITY: Input validation
    if user_principal == Principal::anonymous() {
        return Err("Invalid user principal".to_string());
    }
    
    if chain.is_empty() || asset.is_empty() || tx_hash.is_empty() {
        return Err("Invalid input: chain, asset, and tx_hash cannot be empty".to_string());
    }
    
    if amount <= 0.0 || amount_usd <= 0.0 || !amount.is_finite() || !amount_usd.is_finite() {
        return Err("Invalid amounts: must be positive finite numbers".to_string());
    }
    
    // SECURITY: Prevent duplicate transaction hash
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        for existing_tx in &pool_state.treasury_transactions {
            if let Some(existing_hash) = &existing_tx.tx_hash {
                if existing_hash == &tx_hash {
                    return Err(format!("Transaction hash already recorded: {}", tx_hash));
                }
            }
        }
        Ok(())
    })?;
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Recording subscription payment - User: {}, Amount: ${}, TX: {}", 
                     user_principal.to_text(), amount_usd, tx_hash);
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Get payment address
        let key = format!("{}_{}", chain, asset);
        let to_address = pool_state.treasury_config.payment_addresses
            .get(&key)
            .unwrap_or(&"unknown".to_string()).clone();
        
        // Record treasury transaction
        let tx = TreasuryTransaction {
            id: format!("sub_{}_{}", user_principal.to_text(), current_time),
            transaction_type: TreasuryTransactionType::SubscriptionPayment,
            chain: chain.clone(),
            asset: asset.clone(),
            amount,
            amount_usd,
            from_address: "user_wallet".to_string(),
            to_address,
            tx_hash: Some(tx_hash.clone()),
            status: TransactionStatus::Confirmed,
            timestamp: current_time,
            initiated_by: user_principal,
            notes: Some(format!("Subscription payment for {} tier", subscription_tier)),
        };
        
        // SECURITY: Check storage limits before adding transaction
        if pool_state.treasury_transactions.len() >= pool_state.storage_metrics.max_treasury_transactions {
            ic_cdk::println!("SECURITY: Treasury transactions limit reached - pruning old records");
            prune_old_transactions(&mut pool_state)?;
        }
        
        pool_state.treasury_transactions.push(tx);
        
        // SECURITY: Update treasury balance with validation
        let mut balance_found = false;
        let old_balance_usd = pool_state.treasury_balances
            .iter()
            .find(|b| b.chain == chain && b.asset == asset)
            .map(|b| b.amount_usd)
            .unwrap_or(0.0);
            
        for balance in &mut pool_state.treasury_balances {
            if balance.chain == chain && balance.asset == asset {
                // SECURITY: Verify balance calculations
                let new_amount = balance.amount + amount;
                let new_amount_usd = balance.amount_usd + amount_usd;
                
                if new_amount < 0.0 || new_amount_usd < 0.0 || !new_amount.is_finite() || !new_amount_usd.is_finite() {
                    return Err("SECURITY: Invalid balance calculation".to_string());
                }
                
                balance.amount = new_amount;
                balance.amount_usd = new_amount_usd;
                balance.last_updated = current_time;
                balance.last_tx_hash = Some(tx_hash.clone());
                balance_found = true;
                break;
            }
        }
        
        if !balance_found {
            let new_balance = TreasuryBalance {
                chain,
                asset,
                amount,
                amount_usd,
                last_updated: current_time,
                last_tx_hash: None,
            };
            pool_state.treasury_balances.push(new_balance);
        }
        
        // Process through existing business model
        pool_state.dev_team_business.monthly_subscription_revenue += amount_usd;
        
        Ok(())
    })
}

#[query]
fn get_treasury_balance(chain: String, asset: String) -> Option<TreasuryBalance> {
    require_manager_or_above().ok()?;
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.treasury_balances.iter()
            .find(|b| b.chain == chain && b.asset == asset)
            .cloned()
    })
}

#[query]
fn get_all_treasury_balances() -> Vec<TreasuryBalance> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    POOL_STATE.with(|state| {
        state.borrow().treasury_balances.clone()
    })
}

#[update]
fn request_treasury_withdrawal(
    chain: String,
    asset: String,
    amount: f64,
    destination_address: String,
    reason: String
) -> Result<String, String> {
    let caller = require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Calculate USD value (simplified - in production, use price oracle)
        let amount_usd = estimate_usd_value(&asset, amount);
        
        // Check if amount exceeds hot wallet limit
        let balance_key = format!("{}_{}", chain, asset);
        let threshold = pool_state.treasury_config.hot_wallet_limits
            .get(&balance_key).unwrap_or(&10000.0); // Default $10K limit
        
        let withdrawal_id = format!("withdraw_{}_{}", caller.to_text(), current_time);
        
        let (status, required_approvals) = if amount_usd > *threshold {
            (WithdrawalStatus::PendingApproval, 2) // Requires multi-sig approval
        } else {
            (WithdrawalStatus::Approved, 0) // Auto-approved for small amounts
        };
        
        let withdrawal_request = WithdrawalRequest {
            id: withdrawal_id.clone(),
            requested_by: caller,
            chain,
            asset,
            amount,
            amount_usd,
            destination_address,
            reason,
            status,
            required_approvals,
            current_approvals: if required_approvals == 0 { vec![caller] } else { Vec::new() },
            created_at: current_time,
            approved_at: if required_approvals == 0 { Some(current_time) } else { None },
            executed_at: None,
            tx_hash: None,
        };
        
        pool_state.withdrawal_requests.push(withdrawal_request);
        
        if required_approvals == 0 {
            Ok(format!("Withdrawal {} auto-approved and ready for execution", withdrawal_id))
        } else {
            Ok(format!("Withdrawal {} requires {} approvals", withdrawal_id, required_approvals))
        }
    })
}

#[update]
fn approve_withdrawal(withdrawal_id: String) -> Result<(), String> {
    let caller = require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        if let Some(withdrawal) = pool_state.withdrawal_requests.iter_mut()
            .find(|w| w.id == withdrawal_id) {
            
            if withdrawal.status != WithdrawalStatus::PendingApproval {
                return Err("Withdrawal is not pending approval".to_string());
            }
            
            if withdrawal.current_approvals.contains(&caller) {
                return Err("You have already approved this withdrawal".to_string());
            }
            
            withdrawal.current_approvals.push(caller);
            
            // Check if we have enough approvals
            if withdrawal.current_approvals.len() >= withdrawal.required_approvals as usize {
                withdrawal.status = WithdrawalStatus::Approved;
                withdrawal.approved_at = Some(ic_cdk::api::time());
            }
            
            Ok(())
        } else {
            Err("Withdrawal request not found".to_string())
        }
    })
}

#[query]
fn get_treasury_health_report() -> TreasuryHealthReport {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        // Return limited info for non-managers
        return TreasuryHealthReport {
            total_usd_value: 0.0,
            total_assets: 0,
            balances_over_limit: Vec::new(),
            last_payment_timestamp: None,
            pending_withdrawals: 0,
            hot_wallet_utilization: 0.0,
            largest_single_balance: 0.0,
            diversification_score: 0.0,
            security_alerts: vec!["Access restricted".to_string()],
        };
    }
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        let mut total_usd_value = 0.0;
        let mut balances_over_limit = Vec::new();
        let mut largest_single_balance = 0.0;
        
        for balance in &pool_state.treasury_balances {
            total_usd_value += balance.amount_usd;
            
            if balance.amount_usd > largest_single_balance {
                largest_single_balance = balance.amount_usd;
            }
            
            let key = format!("{}_{}", balance.chain, balance.asset);
            if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
                if balance.amount_usd > *limit {
                    balances_over_limit.push(format!("{}: ${:.2} (limit: ${:.2})", 
                        key, balance.amount_usd, limit));
                }
            }
        }
        
        let last_payment_timestamp = pool_state.treasury_transactions
            .iter()
            .filter(|tx| tx.transaction_type == TreasuryTransactionType::SubscriptionPayment)
            .map(|tx| tx.timestamp)
            .max();
        
        let pending_withdrawals = pool_state.withdrawal_requests
            .iter()
            .filter(|w| w.status == WithdrawalStatus::PendingApproval)
            .count();
        
        let hot_wallet_utilization = calculate_hot_wallet_utilization(&pool_state);
        let diversification_score = calculate_diversification_score(&pool_state.treasury_balances);
        let security_alerts = generate_security_alerts(&pool_state);
        
        TreasuryHealthReport {
            total_usd_value,
            total_assets: pool_state.treasury_balances.len(),
            balances_over_limit,
            last_payment_timestamp,
            pending_withdrawals,
            hot_wallet_utilization,
            largest_single_balance,
            diversification_score,
            security_alerts,
        }
    })
}

#[query]
fn get_treasury_transactions(limit: Option<usize>) -> Vec<TreasuryTransaction> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let mut transactions = pool_state.treasury_transactions.clone();
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            transactions.truncate(limit);
        }
        
        transactions
    })
}

// =============================================================================
// TREASURY UTILITY FUNCTIONS
// =============================================================================

fn estimate_usd_value(asset: &str, amount: f64) -> f64 {
    // SECURITY: Input validation
    if asset.is_empty() || asset.len() > 10 {
        ic_cdk::println!("SECURITY: Invalid asset name: {}", asset);
        return 0.0;
    }
    
    if amount < 0.0 || amount > 1_000_000_000.0 || !amount.is_finite() {
        ic_cdk::println!("SECURITY: Invalid amount: {}", amount);
        return 0.0;
    }
    
    // TODO: Replace with secure price oracle - this is temporary
    let price_multiplier = match asset.to_uppercase().as_str() {
        "USDC" | "USDT" | "DAI" => 1.0, // Stablecoins = 1:1 USD
        "ETH" => 2500.0, // TEMPORARY: Use oracle
        "BTC" => 45000.0, // TEMPORARY: Use oracle
        "SOL" => 100.0, // TEMPORARY: Use oracle
        "MATIC" => 0.9, // TEMPORARY: Use oracle
        _ => {
            ic_cdk::println!("SECURITY: Unknown asset: {}", asset);
            return 0.0; // Don't guess unknown assets
        }
    };
    
    let result = amount * price_multiplier;
    
    // SECURITY: Bounds check result
    if result > 1_000_000_000_000.0 || !result.is_finite() {
        ic_cdk::println!("SECURITY: Calculation overflow for asset: {}, amount: {}", asset, amount);
        return 0.0;
    }
    
    result
}

fn calculate_hot_wallet_utilization(pool_state: &PoolState) -> f64 {
    let mut total_used = 0.0;
    let mut total_limits = 0.0;
    
    for balance in &pool_state.treasury_balances {
        let key = format!("{}_{}", balance.chain, balance.asset);
        if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
            total_used += balance.amount_usd;
            total_limits += limit;
        }
    }
    
    if total_limits > 0.0 {
        (total_used / total_limits) * 100.0
    } else {
        0.0
    }
}

fn calculate_diversification_score(balances: &Vec<TreasuryBalance>) -> f64 {
    if balances.is_empty() {
        return 0.0;
    }
    
    let total_value: f64 = balances.iter().map(|b| b.amount_usd).sum();
    if total_value == 0.0 {
        return 0.0;
    }
    
    // Calculate Herfindahl-Hirschman Index for diversification
    let hhi: f64 = balances.iter()
        .map(|b| {
            let share = b.amount_usd / total_value;
            share * share
        })
        .sum();
    
    // Convert to diversification score (1 = perfectly diversified, 0 = all in one asset)
    1.0 - hhi
}

fn generate_security_alerts(pool_state: &PoolState) -> Vec<String> {
    let mut alerts = Vec::new();
    
    // Check for balances over limits
    for balance in &pool_state.treasury_balances {
        let key = format!("{}_{}", balance.chain, balance.asset);
        if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
            if balance.amount_usd > *limit {
                alerts.push(format!("⚠️ {} balance exceeds limit: ${:.2} > ${:.2}", 
                    key, balance.amount_usd, limit));
            }
        }
    }
    
    // Check for stale balances (not updated in 24 hours)
    let current_time = ic_cdk::api::time();
    let day_in_ns = 24 * 60 * 60 * 1_000_000_000;
    
    for balance in &pool_state.treasury_balances {
        if current_time - balance.last_updated > day_in_ns {
            alerts.push(format!("🕐 Stale balance data for {}_{}", balance.chain, balance.asset));
        }
    }
    
    // Check for pending withdrawals older than 48 hours
    let two_days_in_ns = 2 * day_in_ns;
    
    for withdrawal in &pool_state.withdrawal_requests {
        if withdrawal.status == WithdrawalStatus::PendingApproval && 
           current_time - withdrawal.created_at > two_days_in_ns {
            alerts.push(format!("⏰ Pending withdrawal {} requires attention", withdrawal.id));
        }
    }
    
    if alerts.is_empty() {
        alerts.push("✅ No security alerts".to_string());
    }
    
    alerts
}

// SECURITY: Storage management functions
fn prune_old_transactions(pool_state: &mut PoolState) -> Result<(), String> {
    let initial_count = pool_state.treasury_transactions.len();
    let target_count = pool_state.storage_metrics.max_treasury_transactions * 3 / 4; // Keep 75% of max
    
    if initial_count <= target_count {
        return Ok(()); // No pruning needed
    }
    
    // Sort by timestamp (newest first)
    pool_state.treasury_transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Keep only the most recent transactions
    pool_state.treasury_transactions.truncate(target_count);
    
    let pruned_count = initial_count - pool_state.treasury_transactions.len();
    pool_state.storage_metrics.transactions_pruned += pruned_count as u64;
    pool_state.storage_metrics.last_cleanup_time = ic_cdk::api::time();
    
    ic_cdk::println!("SECURITY: Pruned {} old transactions, kept {}", pruned_count, pool_state.treasury_transactions.len());
    
    Ok(())
}

fn check_storage_limits(pool_state: &PoolState) -> Result<(), String> {
    let metrics = &pool_state.storage_metrics;
    
    if pool_state.treasury_transactions.len() >= metrics.max_treasury_transactions {
        return Err("SECURITY: Treasury transactions storage limit exceeded".to_string());
    }
    
    if pool_state.withdrawal_requests.len() >= metrics.max_withdrawal_requests {
        return Err("SECURITY: Withdrawal requests storage limit exceeded".to_string());
    }
    
    if pool_state.payment_addresses.len() >= metrics.max_payment_addresses {
        return Err("SECURITY: Payment addresses storage limit exceeded".to_string());
    }
    
    Ok(())
}

// =============================================================================
// PAYMENT METHODS API
// =============================================================================

#[query]
fn get_supported_payment_methods() -> Vec<PaymentMethod> {
    vec![
        // Ethereum USDC
        PaymentMethod {
            id: "ethereum_usdc".to_string(),
            chain: ChainId::Ethereum,
            asset: Asset::USDC,
            token_address: Some("0xA0b86a33E6441b5cBb5b9c7e9a8e49A44A2a1c6f".to_string()), // USDC on Ethereum
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 100, // 1%
            confirmation_blocks: 12,
            estimated_settlement_time: 900, // 15 minutes
        },
        // Ethereum USDT
        PaymentMethod {
            id: "ethereum_usdt".to_string(),
            chain: ChainId::Ethereum,
            asset: Asset::USDT,
            token_address: Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()), // USDT on Ethereum
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 100, // 1%
            confirmation_blocks: 12,
            estimated_settlement_time: 900, // 15 minutes
        },
        // Polygon USDC
        PaymentMethod {
            id: "polygon_usdc".to_string(),
            chain: ChainId::Polygon,
            asset: Asset::USDC,
            token_address: Some("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string()), // USDC on Polygon
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 75, // 0.75%
            confirmation_blocks: 20,
            estimated_settlement_time: 300, // 5 minutes
        },
        // Polygon USDT
        PaymentMethod {
            id: "polygon_usdt".to_string(),
            chain: ChainId::Polygon,
            asset: Asset::USDT,
            token_address: Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string()), // USDT on Polygon
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 75, // 0.75%
            confirmation_blocks: 20,
            estimated_settlement_time: 300, // 5 minutes
        },
        // Arbitrum USDC
        PaymentMethod {
            id: "arbitrum_usdc".to_string(),
            chain: ChainId::Arbitrum,
            asset: Asset::USDC,
            token_address: Some("0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8".to_string()), // USDC on Arbitrum
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 50, // 0.5%
            confirmation_blocks: 1,
            estimated_settlement_time: 60, // 1 minute
        },
        // Base USDC
        PaymentMethod {
            id: "base_usdc".to_string(),
            chain: ChainId::Base,
            asset: Asset::USDC,
            token_address: Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()), // USDC on Base
            enabled: true,
            min_amount_usd: 1.0,
            max_amount_usd: 10000.0,
            processing_fee_bps: 50, // 0.5%
            confirmation_blocks: 1,
            estimated_settlement_time: 60, // 1 minute
        },
    ]
}

#[update]
fn create_payment_request(
    payment_method_id: String,
    amount_usd: f64,
    purpose: PaymentPurpose,
    sender_address: String
) -> Result<Payment, String> {
    let caller = ic_cdk::caller();
    let current_time = ic_cdk::api::time();
    
    // SECURITY: Input validation
    if payment_method_id.is_empty() || payment_method_id.len() > 50 {
        return Err("Invalid payment method ID".to_string());
    }
    
    if amount_usd <= 0.0 || amount_usd > 1_000_000.0 || !amount_usd.is_finite() {
        return Err("Invalid amount: Must be between $0.01 and $1,000,000".to_string());
    }
    
    if sender_address.is_empty() || sender_address.len() > 100 {
        return Err("Invalid sender address".to_string());
    }
    
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot create payment requests".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Payment request - Method: {}, Amount: ${}, Caller: {}", 
                     payment_method_id, amount_usd, caller.to_text());
    
    // Find payment method
    let payment_methods = get_supported_payment_methods();
    let payment_method = payment_methods.iter()
        .find(|pm| pm.id == payment_method_id)
        .ok_or("Payment method not found")?;
    
    if !payment_method.enabled {
        return Err("Payment method is currently disabled".to_string());
    }
    
    if amount_usd < payment_method.min_amount_usd || amount_usd > payment_method.max_amount_usd {
        return Err(format!("Amount must be between ${} and ${}", 
                          payment_method.min_amount_usd, payment_method.max_amount_usd));
    }
    
    // Calculate fee
    let fee_amount_usd = amount_usd * (payment_method.processing_fee_bps as f64) / 10000.0;
    let total_amount_usd = amount_usd + fee_amount_usd;
    
    // Convert to token units (assuming 1:1 for stablecoins)
    let amount = total_amount_usd;
    let fee_amount = fee_amount_usd;
    
    // Get destination address from treasury config
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let key = format!("{}_{}", 
                         payment_method.chain.to_string().to_lowercase(),
                         asset_to_string(&payment_method.asset).to_lowercase());
        
        let destination_address = pool_state.treasury_config.payment_addresses
            .get(&key)
            .ok_or("Treasury address not configured for this payment method")?
            .clone();
        
        let payment_id = format!("pay_{}_{}", caller.to_text(), current_time);
        
        let payment = Payment {
            id: payment_id,
            user_principal: caller,
            payment_method: payment_method.clone(),
            amount,
            amount_usd: total_amount_usd,
            fee_amount,
            fee_amount_usd,
            destination_address,
            sender_address,
            tx_hash: None,
            status: PaymentStatus::Created,
            initiated_at: current_time,
            confirmed_at: None,
            expires_at: current_time + (24 * 60 * 60 * 1_000_000_000), // 24 hours in nanoseconds
            purpose,
            metadata: PaymentMetadata {
                invoice_id: None,
                notes: None,
                tags: vec![],
                refund_policy: RefundPolicy::FullRefund { within_hours: 24 },
            },
        };
        
        Ok(payment)
    })
}

#[update]
fn confirm_payment(payment_id: String, tx_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Create treasury transaction for the payment
        let treasury_tx = TreasuryTransaction {
            id: format!("payment_{}", payment_id),
            transaction_type: TreasuryTransactionType::PaymentReceived,
            chain: "polygon".to_string(), // This should come from payment data
            asset: "usdc".to_string(),    // This should come from payment data
            amount: 100.0,                // This should come from payment data
            amount_usd: 100.0,           // This should come from payment data
            from_address: "user_wallet".to_string(), // This should come from payment data
            to_address: "treasury_wallet".to_string(), // This should come from payment data
            tx_hash: Some(tx_hash),
            status: TransactionStatus::Confirmed,
            timestamp: current_time,
            initiated_by: caller,
            notes: Some(format!("Payment confirmed for user {}", caller.to_text())),
        };
        
        pool_state.treasury_transactions.push(treasury_tx);
        Ok(())
    })
}

#[query]
fn get_payment_status(payment_id: String) -> Result<PaymentStatus, String> {
    let caller = ic_cdk::caller();
    
    // In a full implementation, we would store payments and check their status
    // For now, return a mock status
    Ok(PaymentStatus::WaitingConfirmation)
}

#[query]
fn get_user_payments(user_principal: Principal) -> Vec<Payment> {
    let caller = ic_cdk::caller();
    
    // Only allow users to see their own payments, or managers to see all
    if caller != user_principal && !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    // In a full implementation, we would fetch payments from storage
    // For now, return empty list
    Vec::new()
}

#[update]
fn issue_refund(payment_id: String, reason: String) -> Result<(), String> {
    require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Create refund transaction
        let refund_tx = TreasuryTransaction {
            id: format!("refund_{}", payment_id),
            transaction_type: TreasuryTransactionType::RefundIssued,
            chain: "polygon".to_string(), // This should come from original payment
            asset: "usdc".to_string(),    // This should come from original payment
            amount: 100.0,                // This should come from original payment
            amount_usd: 100.0,           // This should come from original payment
            from_address: "treasury_wallet".to_string(),
            to_address: "user_wallet".to_string(), // This should come from original payment
            tx_hash: None, // Will be filled when refund is processed
            status: TransactionStatus::Pending,
            timestamp: current_time,
            initiated_by: ic_cdk::caller(),
            notes: Some(format!("Refund issued: {}", reason)),
        };
        
        pool_state.treasury_transactions.push(refund_tx);
        Ok(())
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