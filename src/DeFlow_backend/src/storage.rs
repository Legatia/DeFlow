use crate::types::{
    Workflow, WorkflowExecution, NodeDefinition, EventListener, 
    ScheduledWorkflow, RetryPolicy
};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static WORKFLOWS: RefCell<HashMap<String, Workflow>> = RefCell::new(HashMap::new());
    pub static EXECUTIONS: RefCell<HashMap<String, WorkflowExecution>> = RefCell::new(HashMap::new());
    pub static NODE_REGISTRY: RefCell<HashMap<String, NodeDefinition>> = RefCell::new(HashMap::new());
    pub static EVENT_LISTENERS: RefCell<HashMap<String, Vec<EventListener>>> = RefCell::new(HashMap::new());
    pub static SCHEDULED_WORKFLOWS: RefCell<HashMap<String, ScheduledWorkflow>> = RefCell::new(HashMap::new());
    pub static TIMERS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    pub static WEBHOOK_ENDPOINTS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    pub static RETRY_POLICIES: RefCell<HashMap<String, RetryPolicy>> = RefCell::new(HashMap::new());
}