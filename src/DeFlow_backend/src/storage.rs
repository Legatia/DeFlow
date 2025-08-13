use crate::types::{
    Workflow, WorkflowExecution, NodeDefinition, EventListener, 
    ScheduledWorkflow, RetryPolicy, WorkflowState, ScheduledExecution
};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Encode, Decode};
use serde::Serialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// Wrapper types for stable storage
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableWorkflow(pub Workflow);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableExecution(pub WorkflowExecution);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableNodeDefinition(pub NodeDefinition);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableEventListeners(pub Vec<EventListener>);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableScheduledWorkflow(pub ScheduledWorkflow);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableRetryPolicy(pub RetryPolicy);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableWorkflowState(pub WorkflowState);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableScheduledExecution(pub ScheduledExecution);

// Implement Storable trait for our wrapper types
impl ic_stable_structures::Storable for StorableWorkflow {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 8192,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableExecution {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 16384,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableNodeDefinition {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 4096,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableEventListeners {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 8192,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableScheduledWorkflow {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 2048,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableRetryPolicy {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 1024,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableWorkflowState {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 65536, // Large size for comprehensive state
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl ic_stable_structures::Storable for StorableScheduledExecution {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 4096,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static WORKFLOWS: RefCell<StableBTreeMap<String, StorableWorkflow, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static EXECUTIONS: RefCell<StableBTreeMap<String, StorableExecution, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    pub static NODE_REGISTRY: RefCell<StableBTreeMap<String, StorableNodeDefinition, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    pub static EVENT_LISTENERS: RefCell<StableBTreeMap<String, StorableEventListeners, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    pub static SCHEDULED_WORKFLOWS: RefCell<StableBTreeMap<String, StorableScheduledWorkflow, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
        )
    );

    pub static RETRY_POLICIES: RefCell<StableBTreeMap<String, StorableRetryPolicy, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
        )
    );

    // Global workflow state for zero-downtime architecture
    pub static WORKFLOW_STATE: RefCell<StableBTreeMap<String, StorableWorkflowState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
        )
    );

    // Persistent scheduled executions that survive canister upgrades
    pub static SCHEDULED_EXECUTIONS: RefCell<StableBTreeMap<String, StorableScheduledExecution, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
        )
    );

    // Keep these as thread-local for temporary data
    pub static TIMERS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    pub static WEBHOOK_ENDPOINTS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

// Helper functions for accessing stable storage
pub fn get_workflow(id: &str) -> Option<Workflow> {
    WORKFLOWS.with(|workflows| {
        workflows.borrow().get(&id.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_workflow(id: String, workflow: Workflow) {
    WORKFLOWS.with(|workflows| {
        workflows.borrow_mut().insert(id, StorableWorkflow(workflow));
    });
}

pub fn remove_workflow(id: &str) -> Option<Workflow> {
    WORKFLOWS.with(|workflows| {
        workflows.borrow_mut().remove(&id.to_string()).map(|storable| storable.0)
    })
}

pub fn get_execution(id: &str) -> Option<WorkflowExecution> {
    EXECUTIONS.with(|executions| {
        executions.borrow().get(&id.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_execution(id: String, execution: WorkflowExecution) {
    EXECUTIONS.with(|executions| {
        executions.borrow_mut().insert(id, StorableExecution(execution));
    });
}

pub fn get_node_definition(node_type: &str) -> Option<NodeDefinition> {
    NODE_REGISTRY.with(|registry| {
        registry.borrow().get(&node_type.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_node_definition(node_type: String, definition: NodeDefinition) {
    NODE_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(node_type, StorableNodeDefinition(definition));
    });
}

pub fn get_event_listeners(event_type: &str) -> Vec<EventListener> {
    EVENT_LISTENERS.with(|listeners| {
        listeners.borrow().get(&event_type.to_string())
            .map(|storable| storable.0.clone())
            .unwrap_or_default()
    })
}

pub fn insert_event_listener(event_type: String, listener: EventListener) {
    EVENT_LISTENERS.with(|listeners| {
        let mut borrowed = listeners.borrow_mut();
        let mut current_listeners = borrowed.get(&event_type)
            .map(|storable| storable.0.clone())
            .unwrap_or_default();
        current_listeners.push(listener);
        borrowed.insert(event_type, StorableEventListeners(current_listeners));
    });
}

pub fn get_scheduled_workflow(id: &str) -> Option<ScheduledWorkflow> {
    SCHEDULED_WORKFLOWS.with(|schedules| {
        schedules.borrow().get(&id.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_scheduled_workflow(id: String, schedule: ScheduledWorkflow) {
    SCHEDULED_WORKFLOWS.with(|schedules| {
        schedules.borrow_mut().insert(id, StorableScheduledWorkflow(schedule));
    });
}

pub fn remove_scheduled_workflow(id: &str) -> Option<ScheduledWorkflow> {
    SCHEDULED_WORKFLOWS.with(|schedules| {
        schedules.borrow_mut().remove(&id.to_string()).map(|storable| storable.0)
    })
}

pub fn get_retry_policy(node_type: &str) -> Option<RetryPolicy> {
    RETRY_POLICIES.with(|policies| {
        policies.borrow().get(&node_type.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_retry_policy(node_type: String, policy: RetryPolicy) {
    RETRY_POLICIES.with(|policies| {
        policies.borrow_mut().insert(node_type, StorableRetryPolicy(policy));
    });
}

// WorkflowState management functions
pub fn get_workflow_state() -> WorkflowState {
    WORKFLOW_STATE.with(|state| {
        state.borrow().get(&"global".to_string())
            .map(|storable| storable.0)
            .unwrap_or_default()
    })
}

pub fn update_workflow_state(new_state: WorkflowState) {
    WORKFLOW_STATE.with(|state| {
        state.borrow_mut().insert("global".to_string(), StorableWorkflowState(new_state));
    });
}

pub fn save_workflow_state_for_upgrade() -> WorkflowState {
    get_workflow_state()
}

pub fn restore_workflow_state_after_upgrade(state: WorkflowState) {
    update_workflow_state(state);
}

// Persistent scheduled execution management
pub fn get_scheduled_execution(workflow_id: &str) -> Option<ScheduledExecution> {
    SCHEDULED_EXECUTIONS.with(|executions| {
        executions.borrow().get(&workflow_id.to_string())
            .map(|storable| storable.0)
    })
}

pub fn insert_scheduled_execution(workflow_id: String, execution: ScheduledExecution) {
    SCHEDULED_EXECUTIONS.with(|executions| {
        executions.borrow_mut().insert(workflow_id, StorableScheduledExecution(execution));
    });
}

pub fn remove_scheduled_execution(workflow_id: &str) -> Option<ScheduledExecution> {
    SCHEDULED_EXECUTIONS.with(|executions| {
        executions.borrow_mut().remove(&workflow_id.to_string())
            .map(|storable| storable.0)
    })
}

pub fn list_all_scheduled_executions() -> Vec<(String, ScheduledExecution)> {
    SCHEDULED_EXECUTIONS.with(|executions| {
        executions.borrow().iter()
            .map(|(id, storable)| (id, storable.0))
            .collect()
    })
}

#[allow(dead_code)]
pub fn get_due_scheduled_executions(current_time: u64) -> Vec<(String, ScheduledExecution)> {
    SCHEDULED_EXECUTIONS.with(|executions| {
        executions.borrow().iter()
            .filter(|(_, storable)| storable.0.next_execution <= current_time)
            .map(|(id, storable)| (id, storable.0))
            .collect()
    })
}