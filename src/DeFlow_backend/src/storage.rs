use crate::types::{
    Workflow, WorkflowExecution, NodeDefinition, EventListener, 
    ScheduledWorkflow, RetryPolicy
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