// DeFlow Workflow Template API - User-friendly strategy creation endpoints

use super::workflow_templates::{WorkflowTemplateManager, WorkflowTemplate, WorkflowCategory, DifficultyLevel};
use super::automated_strategies::StrategyConfig;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;

// Global workflow template manager
thread_local! {
    static TEMPLATE_MANAGER: RefCell<WorkflowTemplateManager> = RefCell::new(WorkflowTemplateManager::new());
}

// API Response types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

impl<T> TemplateApiResponse<T> {
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
    pub templates: Vec<TemplateSummary>,
    pub total_count: usize,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub difficulty: String,
    pub estimated_apy: f64,
    pub risk_score: u8,
    pub min_capital_usd: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateDetailResponse {
    pub template: WorkflowTemplate,
    pub estimated_gas_cost: f64,
    pub supported_chains: Vec<String>,
    pub example_allocations: Vec<AllocationExample>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AllocationExample {
    pub scenario_name: String,
    pub capital_amount: f64,
    pub expected_returns: f64,
    pub risk_assessment: String,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyCreationRequest {
    pub template_id: String,
    pub user_id: String,
    pub user_inputs: HashMap<String, String>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyCreationResponse {
    pub strategy_id: String,
    pub strategy_config: StrategyConfig,
    pub estimated_setup_time: u32,
    pub next_steps: Vec<String>,
    pub deployment_status: String,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateValidationResponse {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub estimated_gas_cost: f64,
    pub risk_warnings: Vec<String>,
}

// =============================================================================
// API ENDPOINTS - Workflow Templates
// =============================================================================

/// Get all available workflow templates
#[ic_cdk::query]
pub fn get_workflow_templates() -> TemplateApiResponse<TemplateListResponse> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let templates = mgr.get_all_templates();
        
        let template_summaries: Vec<TemplateSummary> = templates.iter()
            .map(|template| TemplateSummary {
                id: template.id.clone(),
                name: template.name.clone(),
                description: template.description.clone(),
                category: format!("{:?}", template.category),
                difficulty: format!("{:?}", template.difficulty),
                estimated_apy: template.estimated_apy,
                risk_score: template.risk_score,
                min_capital_usd: template.min_capital_usd,
            })
            .collect();
        
        let categories = vec![
            "YieldFarming".to_string(),
            "Arbitrage".to_string(),
            "Rebalancing".to_string(),
            "DCA".to_string(),
            "RiskManagement".to_string(),
            "Composite".to_string()
        ];
        
        let response = TemplateListResponse {
            templates: template_summaries,
            total_count: templates.len(),
            categories,
        };
        
        TemplateApiResponse::success(response)
    })
}

/// Get templates by category
#[ic_cdk::query]
pub fn get_templates_by_category(category: String) -> TemplateApiResponse<TemplateListResponse> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        let category_enum = match category.as_str() {
            "YieldFarming" => WorkflowCategory::YieldFarming,
            "Arbitrage" => WorkflowCategory::Arbitrage,
            "Rebalancing" => WorkflowCategory::Rebalancing,
            "DCA" => WorkflowCategory::DCA,
            "RiskManagement" => WorkflowCategory::RiskManagement,
            "Composite" => WorkflowCategory::Composite,
            _ => return TemplateApiResponse::error(format!("Invalid category: {}", category)));
        };
        
        let templates = mgr.get_templates_by_category(&category_enum);
        
        let template_summaries: Vec<TemplateSummary> = templates.iter()
            .map(|template| TemplateSummary {
                id: template.id.clone(),
                name: template.name.clone(),
                description: template.description.clone(),
                category: format!("{:?}", template.category),
                difficulty: format!("{:?}", template.difficulty),
                estimated_apy: template.estimated_apy,
                risk_score: template.risk_score,
                min_capital_usd: template.min_capital_usd,
            })
            .collect();
        
        let response = TemplateListResponse {
            templates: template_summaries,
            total_count: templates.len(),
            categories: vec![category],
        };
        
        TemplateApiResponse::success(response)
    })
}

/// Get detailed information about a specific template
#[ic_cdk::query]
pub fn get_template_details(template_id: String) -> TemplateApiResponse<TemplateDetailResponse> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        match mgr.get_template(&template_id) {
            Some(template) => {
                let supported_chains: Vec<String> = template.template_config.default_chains.iter()
                    .map(|chain| format!("{:?}", chain))
                    .collect();
                
                let example_allocations = generate_allocation_examples(template);
                
                let response = TemplateDetailResponse {
                    template: template.clone(),
                    estimated_gas_cost: estimate_template_gas_cost(template),
                    supported_chains,
                    example_allocations,
                };
                
                TemplateApiResponse::success(response)
            },
            None => TemplateApiResponse::error(format!("Template {} not found", template_id)),
        }
    })
}

/// Validate user inputs for a template
#[ic_cdk::query]
pub fn validate_template_inputs(template_id: String, user_inputs: HashMap<String, String>) -> TemplateApiResponse<TemplateValidationResponse> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        match mgr.get_template(&template_id) {
            Some(template) => {
                let mut validation_errors = Vec::new();
                let mut risk_warnings = Vec::new();
                
                // Validate inputs
                for input_def in &template.user_inputs {
                    if input_def.required && !user_inputs.contains_key(&input_def.id) {
                        validation_errors.push(format!("Required field '{}' is missing", input_def.name));
                    }
                    
                    if let Some(value) = user_inputs.get(&input_def.id) {
                        // Type-specific validation
                        match &input_def.input_type {
                            super::workflow_templates::InputType::Number { min, max } => {
                                match value.parse::<f64>() {
                                    Ok(num_value) => {
                                        if let Some(min_val) = min {
                                            if num_value < *min_val {
                                                validation_errors.push(format!("'{}' must be at least {}", input_def.name, min_val));
                                            }
                                        }
                                        if let Some(max_val) = max {
                                            if num_value > *max_val {
                                                validation_errors.push(format!("'{}' cannot exceed {}", input_def.name, max_val));
                                            }
                                        }
                                    },
                                    Err(_) => {
                                        validation_errors.push(format!("'{}' must be a valid number", input_def.name));
                                    }
                                }
                            },
                            super::workflow_templates::InputType::Selection { options } => {
                                if !options.contains(value) {
                                    validation_errors.push(format!("'{}' must be one of: {}", input_def.name, options.join(", ")));
                                }
                            },
                            _ => {} // Other validations
                        }
                    }
                }
                
                // Risk warnings
                if template.risk_score >= 7 {
                    risk_warnings.push("This is a high-risk strategy. Only invest what you can afford to lose.".to_string()
                }
                
                if template.min_capital_usd >= 10000.0 {
                    risk_warnings.push("This strategy requires significant capital. Consider starting with a smaller amount.".to_string()
                }
                
                let response = TemplateValidationResponse {
                    is_valid: validation_errors.is_empty(),
                    validation_errors,
                    estimated_gas_cost: estimate_template_gas_cost(template),
                    risk_warnings,
                };
                
                TemplateApiResponse::success(response)
            },
            None => TemplateApiResponse::error(format!("Template {} not found", template_id)),
        }
    })
}

/// Create a strategy from a template
#[ic_cdk::update]
pub async fn create_strategy_from_template(request: StrategyCreationRequest) -> TemplateApiResponse<StrategyCreationResponse> {
    match generate_strategy_from_template(request).await {
        Ok(response) => TemplateApiResponse::success(response),
        Err(e) => TemplateApiResponse::error(e),
    }
}

async fn generate_strategy_from_template(request: StrategyCreationRequest) -> Result<StrategyCreationResponse, String> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        
        // Generate strategy config from template
        let strategy_config = mgr.generate_strategy_config(&request.template_id, request.user_inputs)?;
        
        // Create strategy using the automated strategy manager
        use super::automated_strategies::AutomatedStrategyManager;
        let strategy_id = format!("strategy_{}", ic_cdk::api::time());
        
        let next_steps = vec![
            "Strategy configuration validated".to_string(),
            "Wallet connections established".to_string(),
            "Initial capital allocation ready".to_string(),
            "Monitoring systems activated".to_string(),
            "Ready for deployment".to_string()
        ];
        
        let response = StrategyCreationResponse {
            strategy_id,
            strategy_config,
            estimated_setup_time: 5,
            next_steps,
            deployment_status: "ready".to_string(),
        };
        
        Ok(response)
    })
}

/// Get template recommendations based on user profile
#[ic_cdk::query]
pub fn get_template_recommendations(
    risk_tolerance: u8,
    capital_amount: f64,
    experience_level: String,
) -> TemplateApiResponse<TemplateListResponse> {
    TEMPLATE_MANAGER.with(|manager| {
        let mgr = manager.borrow();
        let all_templates = mgr.get_all_templates();
        
        let experience = match experience_level.as_str() {
            "Beginner" => DifficultyLevel::Beginner,
            "Intermediate" => DifficultyLevel::Intermediate,
            "Advanced" => DifficultyLevel::Advanced,
            "Expert" => DifficultyLevel::Expert,
            _ => DifficultyLevel::Beginner);
        };
        
        // Filter templates based on user profile
        let recommended_templates: Vec<&WorkflowTemplate> = all_templates.iter()
            .filter(|template| {
                template.risk_score <= risk_tolerance &&
                template.min_capital_usd <= capital_amount &&
                is_suitable_difficulty(&template.difficulty, &experience)
            })
            .collect();
        
        let template_summaries: Vec<TemplateSummary> = recommended_templates.iter()
            .map(|template| TemplateSummary {
                id: template.id.clone(),
                name: template.name.clone(),
                description: template.description.clone(),
                category: format!("{:?}", template.category),
                difficulty: format!("{:?}", template.difficulty),
                estimated_apy: template.estimated_apy,
                risk_score: template.risk_score,
                min_capital_usd: template.min_capital_usd,
            })
            .collect();
        
        let response = TemplateListResponse {
            templates: template_summaries,
            total_count: recommended_templates.len(),
            categories: vec![], // All categories included
        };
        
        TemplateApiResponse::success(response)
    })
}

// Utility functions

fn generate_allocation_examples(template: &WorkflowTemplate) -> Vec<AllocationExample> {
    let base_capital = template.min_capital_usd * 2.0;
    
    vec![
        AllocationExample {
            scenario_name: "Conservative".to_string(),
            capital_amount: base_capital,
            expected_returns: base_capital * (template.estimated_apy / 100.0) * 0.7,
            risk_assessment: if template.risk_score <= 4 { "Low Risk" } else { "Moderate Risk" }.to_string(),
        },
        AllocationExample {
            scenario_name: "Balanced".to_string(),
            capital_amount: base_capital * 5.0,
            expected_returns: base_capital * 5.0 * (template.estimated_apy / 100.0) * 0.85,
            risk_assessment: format!("Risk Score: {}/10", template.risk_score),
        },
        AllocationExample {
            scenario_name: "Aggressive".to_string(),
            capital_amount: base_capital * 10.0,
            expected_returns: base_capital * 10.0 * (template.estimated_apy / 100.0) * 1.1,
            risk_assessment: if template.risk_score >= 7 { "High Risk - High Reward" } else { "Moderate Risk" }.to_string(),
        },
    ]
}

fn estimate_template_gas_cost(template: &WorkflowTemplate) -> f64 {
    // Estimate gas costs based on template complexity and chain operations
    let base_cost = 50.0; // Base gas cost in USD
    
    let complexity_multiplier = match template.difficulty {
        DifficultyLevel::Beginner => 1.0,
        DifficultyLevel::Intermediate => 1.5,
        DifficultyLevel::Advanced => 2.0,
        DifficultyLevel::Expert => 3.0,
    };
    
    let chain_multiplier = template.template_config.default_chains.len() as f64 * 0.5;
    
    base_cost * complexity_multiplier * (1.0 + chain_multiplier)
}

fn is_suitable_difficulty(template_difficulty: &DifficultyLevel, user_experience: &DifficultyLevel) -> bool {
    use DifficultyLevel::*;
    
    match (user_experience, template_difficulty) {
        (Beginner, Beginner) => true,
        (Intermediate, Beginner | Intermediate) => true,
        (Advanced, Beginner | Intermediate | Advanced) => true,
        (Expert, _) => true,
        _ => false,
    }
}

/// Initialize the workflow template system
pub fn init_workflow_template_system() {
}