// DeFlow Simple Template API - User-friendly strategy creation endpoints

use super::simple_workflow_templates::{SimpleWorkflowTemplateManager, SimpleWorkflowTemplate};
use super::automated_strategies::StrategyConfig;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;

// Global template manager
thread_local! {
    static SIMPLE_TEMPLATE_MANAGER: RefCell<SimpleWorkflowTemplateManager> = RefCell::new(SimpleWorkflowTemplateManager::new());
}

// API Response types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SimpleApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> SimpleApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: ic_cdk::api::time(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: ic_cdk::api::time(),
        }
    }
}

// API Data Transfer Objects
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateListResponse {
    pub templates: Vec<SimpleWorkflowTemplate>,
    pub total_count: usize,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyFromTemplateRequest {
    pub template_id: String,
    pub user_id: String,
    pub capital_amount: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyFromTemplateResponse {
    pub strategy_id: String,
    pub strategy_config: StrategyConfig,
    pub estimated_setup_time: u32,
    pub deployment_status: String,
}

// =============================================================================
// API ENDPOINTS - Simple Workflow Templates
// =============================================================================

/// Get all available workflow templates
#[ic_cdk::query]
pub fn list_workflow_templates() -> SimpleApiResponse<TemplateListResponse> {
    SIMPLE_TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let templates: Vec<SimpleWorkflowTemplate> = mgr.get_all_templates()
            .into_iter()
            .cloned()
            .collect();
        
        let response = TemplateListResponse {
            total_count: templates.len(),
            templates,
        };
        
        SimpleApiResponse::success(response)
    })
}

/// Get templates by category
#[ic_cdk::query]
pub fn get_templates_by_category(category: String) -> SimpleApiResponse<TemplateListResponse> {
    SIMPLE_TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let templates: Vec<SimpleWorkflowTemplate> = mgr.get_templates_by_category(&category)
            .into_iter()
            .cloned()
            .collect();
        
        let response = TemplateListResponse {
            total_count: templates.len(),
            templates,
        };
        
        SimpleApiResponse::success(response)
    })
}

/// Get a specific template by ID
#[ic_cdk::query]
pub fn get_template_by_id(template_id: String) -> SimpleApiResponse<SimpleWorkflowTemplate> {
    SIMPLE_TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        match mgr.get_template(&template_id) {
            Some(template) => SimpleApiResponse::success(template.clone()),
            None => SimpleApiResponse::error(format!("Template {} not found", template_id)),
        }
    })
}

/// Create a strategy from a template
#[ic_cdk::update]
pub async fn create_strategy_from_simple_template(request: StrategyFromTemplateRequest) -> SimpleApiResponse<StrategyFromTemplateResponse> {
    match create_strategy_from_template_impl(request).await {
        Ok(response) => SimpleApiResponse::success(response),
        Err(e) => SimpleApiResponse::error(e),
    }
}

async fn create_strategy_from_template_impl(request: StrategyFromTemplateRequest) -> Result<StrategyFromTemplateResponse, String> {
    // Generate strategy config from template
    let strategy_config = SIMPLE_TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        mgr.generate_strategy_config(&request.template_id, request.capital_amount)
    })?;
    
    // Generate strategy ID (in a real implementation, this would create the strategy)
    let strategy_id = format!("template_strategy_{}", ic_cdk::api::time());
    
    let response = StrategyFromTemplateResponse {
        strategy_id,
        strategy_config,
        estimated_setup_time: 5,
        deployment_status: "created".to_string(),
    };
    
    Ok(response)
}

/// Get template recommendations based on user profile
#[ic_cdk::query]
pub fn get_simple_template_recommendations(
    risk_tolerance: u8,
    capital_amount: f64,
    experience_level: String,
) -> SimpleApiResponse<TemplateListResponse> {
    SIMPLE_TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let all_templates = mgr.get_all_templates();
        
        // Simple filtering based on user profile
        let recommended_templates: Vec<SimpleWorkflowTemplate> = all_templates.iter()
            .filter(|template| {
                template.risk_score <= risk_tolerance &&
                template.min_capital_usd <= capital_amount &&
                is_suitable_for_experience(&template.difficulty, &experience_level)
            })
            .map(|template| (*template).clone())
            .collect();
        
        let response = TemplateListResponse {
            total_count: recommended_templates.len(),
            templates: recommended_templates,
        };
        
        SimpleApiResponse::success(response)
    })
}

/// Get available template categories
#[ic_cdk::query]
pub fn get_template_categories() -> SimpleApiResponse<Vec<String>> {
    let categories = vec![
        "YieldFarming".to_string(),
        "Arbitrage".to_string(),
        "Rebalancing".to_string(),
        "DCA".to_string(),
    ];
    
    SimpleApiResponse::success(categories)
}

// Utility functions

fn is_suitable_for_experience(template_difficulty: &str, user_experience: &str) -> bool {
    match (user_experience, template_difficulty) {
        ("Beginner", "Beginner") => true,
        ("Intermediate", "Beginner" | "Intermediate") => true,
        ("Advanced", _) => true,
        ("Expert", _) => true,
        _ => false,
    }
}

/// Initialize the simple workflow template system
pub fn init_simple_workflow_template_system() {
    SIMPLE_TEMPLATE_MANAGER.with(|_manager| {
        // Template manager is already initialized via lazy static
    });
    ic_cdk::println!("DeFlow Simple Workflow Template System initialized successfully");
}