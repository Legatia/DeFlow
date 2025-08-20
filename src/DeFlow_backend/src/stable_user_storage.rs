// Stable storage for user data - CRITICAL for cross-device persistence!
use crate::types::{
    User, UserSubscriptionInfo, UserSettings,
    IntegrationCredentials, OAuthToken, APIConnection, WorkflowTemplate
};
use ic_stable_structures::memory_manager::{MemoryId, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use candid::{CandidType, Deserialize, Encode, Decode};
use serde::Serialize;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// Wrapper types for stable storage - USER DATA
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableUserProfile(pub User);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableUserSubscriptionInfo(pub UserSubscriptionInfo);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableIntegrationCredentials(pub IntegrationCredentials);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableOAuthToken(pub OAuthToken);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableAPIConnection(pub APIConnection);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableTemplate(pub WorkflowTemplate);

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct StorableUserSettings(pub UserSettings);

// Implement Storable traits for all wrapper types
impl ic_stable_structures::Storable for StorableUserProfile {
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

impl ic_stable_structures::Storable for StorableUserSubscriptionInfo {
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

impl ic_stable_structures::Storable for StorableIntegrationCredentials {
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

impl ic_stable_structures::Storable for StorableOAuthToken {
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

impl ic_stable_structures::Storable for StorableAPIConnection {
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

impl ic_stable_structures::Storable for StorableTemplate {
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

impl ic_stable_structures::Storable for StorableUserSettings {
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

// Import MEMORY_MANAGER from storage.rs
use crate::storage::MEMORY_MANAGER;

thread_local! {
    // User data storage
    pub static USER_PROFILES: RefCell<StableBTreeMap<String, StorableUserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
        )
    );

    pub static USER_SUBSCRIPTION_INFO: RefCell<StableBTreeMap<String, StorableUserSubscriptionInfo, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
        )
    );

    // Integration credentials storage
    pub static USER_INTEGRATIONS: RefCell<StableBTreeMap<String, StorableIntegrationCredentials, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
        )
    );

    pub static OAUTH_TOKENS: RefCell<StableBTreeMap<String, StorableOAuthToken, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))),
        )
    );

    pub static API_CONNECTIONS: RefCell<StableBTreeMap<String, StorableAPIConnection, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12))),
        )
    );

    // Template storage
    pub static GLOBAL_TEMPLATES: RefCell<StableBTreeMap<String, StorableTemplate, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13))),
        )
    );

    pub static USER_TEMPLATES: RefCell<StableBTreeMap<String, StorableTemplate, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14))),
        )
    );

    // User settings storage
    pub static USER_SETTINGS: RefCell<StableBTreeMap<String, StorableUserSettings, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(15))),
        )
    );
}

// Helper functions for user data management
pub fn get_user_profile(principal: &str) -> Option<User> {
    USER_PROFILES.with(|profiles| {
        profiles.borrow().get(&principal.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_user_profile(principal: String, user: User) {
    USER_PROFILES.with(|profiles| {
        profiles.borrow_mut().insert(principal, StorableUserProfile(user));
    });
}

pub fn get_user_subscription_info(principal: &str) -> Option<UserSubscriptionInfo> {
    USER_SUBSCRIPTION_INFO.with(|info| {
        info.borrow().get(&principal.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_user_subscription_info(principal: String, info: UserSubscriptionInfo) {
    USER_SUBSCRIPTION_INFO.with(|subscription_info| {
        subscription_info.borrow_mut().insert(principal, StorableUserSubscriptionInfo(info));
    });
}

pub fn get_user_integration_credentials(user_principal: &str, integration_type: &str) -> Option<IntegrationCredentials> {
    let key = format!("{}:{}", user_principal, integration_type);
    USER_INTEGRATIONS.with(|integrations| {
        integrations.borrow().get(&key).map(|storable| storable.0)
    })
}

pub fn insert_user_integration_credentials(user_principal: String, integration_type: String, credentials: IntegrationCredentials) {
    let key = format!("{}:{}", user_principal, integration_type);
    USER_INTEGRATIONS.with(|integrations| {
        integrations.borrow_mut().insert(key, StorableIntegrationCredentials(credentials));
    });
}

pub fn get_oauth_token(user_principal: &str, platform: &str) -> Option<OAuthToken> {
    let key = format!("{}:{}", user_principal, platform);
    OAUTH_TOKENS.with(|tokens| {
        tokens.borrow().get(&key).map(|storable| storable.0)
    })
}

pub fn insert_oauth_token(user_principal: String, platform: String, token: OAuthToken) {
    let key = format!("{}:{}", user_principal, platform);
    OAUTH_TOKENS.with(|tokens| {
        tokens.borrow_mut().insert(key, StorableOAuthToken(token));
    });
}

pub fn get_user_templates(user_principal: &str) -> Vec<WorkflowTemplate> {
    USER_TEMPLATES.with(|templates| {
        templates.borrow().iter()
            .filter(|(_, template)| template.0.creator_principal == user_principal)
            .map(|(_, template)| template.0)
            .collect()
    })
}

pub fn get_public_templates() -> Vec<WorkflowTemplate> {
    GLOBAL_TEMPLATES.with(|templates| {
        templates.borrow().iter()
            .filter(|(_, template)| template.0.is_public)
            .map(|(_, template)| template.0)
            .collect()
    })
}

pub fn get_user_settings(user_principal: &str) -> Option<UserSettings> {
    USER_SETTINGS.with(|settings| {
        settings.borrow().get(&user_principal.to_string()).map(|storable| storable.0)
    })
}

pub fn insert_user_settings(user_principal: String, settings: UserSettings) {
    USER_SETTINGS.with(|user_settings| {
        user_settings.borrow_mut().insert(user_principal, StorableUserSettings(settings));
    });
}

pub fn insert_template(template_id: String, template: WorkflowTemplate) {
    if template.is_public {
        GLOBAL_TEMPLATES.with(|templates| {
            templates.borrow_mut().insert(template_id.clone(), StorableTemplate(template.clone()));
        });
    }
    USER_TEMPLATES.with(|templates| {
        templates.borrow_mut().insert(template_id, StorableTemplate(template));
    });
}

pub fn get_api_connections(user_principal: &str) -> Vec<APIConnection> {
    API_CONNECTIONS.with(|connections| {
        connections.borrow().iter()
            .filter(|(_, connection)| connection.0.user_principal == user_principal)
            .map(|(_, connection)| connection.0)
            .collect()
    })
}

pub fn insert_api_connection(connection_id: String, connection: APIConnection) {
    API_CONNECTIONS.with(|connections| {
        connections.borrow_mut().insert(connection_id, StorableAPIConnection(connection));
    });
}

// Workflow ownership functions
pub fn get_user_workflows(user_principal: &str) -> Vec<(String, crate::types::Workflow)> {
    crate::storage::WORKFLOWS.with(|workflows| {
        workflows.borrow().iter()
            .filter(|(_, workflow)| workflow.0.owner.as_ref() == Some(&user_principal.to_string()))
            .map(|(id, workflow)| (id, workflow.0))
            .collect()
    })
}

pub fn get_user_executions(user_principal: &str) -> Vec<(String, crate::types::WorkflowExecution)> {
    // This would need workflow ownership lookup first
    let user_workflow_ids: Vec<String> = get_user_workflows(user_principal)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    
    crate::storage::EXECUTIONS.with(|executions| {
        executions.borrow().iter()
            .filter(|(_, execution)| user_workflow_ids.contains(&execution.0.workflow_id))
            .map(|(id, execution)| (id, execution.0))
            .collect()
    })
}