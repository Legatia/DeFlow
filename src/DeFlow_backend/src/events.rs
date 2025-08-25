use crate::types::{
    EventListener, ScheduledWorkflow, WebhookEvent, WorkflowEvent, 
    ConfigValue, RetryPolicy, ScheduledExecution, ScheduleType
};
use crate::storage;
use crate::execution::start_execution;
use crate::workflow::generate_id;
use ic_cdk::{api, update, query, spawn};
use ic_cdk_timers::set_timer;
use std::collections::HashMap;
use std::time::Duration;

// Event System
#[update]
pub async fn emit_event(event: WorkflowEvent) -> Result<Vec<String>, String> {
    let triggered_executions = process_event(&event).await?;
    Ok(triggered_executions)
}

#[update]
pub async fn register_event_listener(listener: EventListener) -> Result<(), String> {
    storage::insert_event_listener(listener.event_type.clone(), listener);
    Ok(())
}

#[update]
pub async fn webhook_trigger(path: String, event: WebhookEvent) -> Result<String, String> {
    let workflow_id = storage::WEBHOOK_ENDPOINTS.with(|endpoints| {
        endpoints.borrow().get(&path).cloned()
    }).ok_or("Webhook endpoint not found")?;
    
    let execution_id = start_execution(workflow_id, Some(event.data)).await?;
    Ok(execution_id)
}

#[update]
pub async fn register_webhook(workflow_id: String, path: String) -> Result<(), String> {
    storage::WEBHOOK_ENDPOINTS.with(|endpoints| {
        endpoints.borrow_mut().insert(path, workflow_id);
    });
    Ok(())
}

// Scheduling System
#[update]
pub async fn schedule_workflow(workflow_id: String, cron_expression: String) -> Result<String, String> {
    let schedule_id = generate_id();
    let next_execution = calculate_next_execution(&cron_expression)?;
    
    let scheduled_workflow = ScheduledWorkflow {
        id: schedule_id.clone(),
        workflow_id: workflow_id.clone(),
        cron_expression: cron_expression.clone(),
        next_execution,
        active: true,
        timer_id: None,
    };
    
    let timer_id = setup_schedule_timer(&scheduled_workflow).await?;
    
    let mut schedule = scheduled_workflow;
    schedule.timer_id = Some(timer_id.clone());
    storage::insert_scheduled_workflow(schedule_id.clone(), schedule);
    
    storage::TIMERS.with(|timers| {
        timers.borrow_mut().insert(schedule_id.clone(), timer_id);
    });
    
    Ok(schedule_id)
}

#[update]
pub async fn unschedule_workflow(schedule_id: String) -> Result<(), String> {
    if let Some(schedule) = storage::remove_scheduled_workflow(&schedule_id) {
        if let Some(timer_id_str) = schedule.timer_id {
            storage::TIMERS.with(|timers| {
                timers.borrow_mut().remove(&timer_id_str);
            });
        }
    }
    Ok(())
}

#[query]
pub fn list_scheduled_workflows() -> Vec<ScheduledWorkflow> {
    storage::SCHEDULED_WORKFLOWS.with(|schedules| {
        schedules.borrow().iter()
            .map(|(_, storable)| storable.0.clone())
            .collect()
    })
}

// Retry Policy Management
#[update]
pub async fn set_retry_policy(node_type: String, policy: RetryPolicy) -> Result<(), String> {
    storage::insert_retry_policy(node_type, policy);
    Ok(())
}

#[query]
pub fn get_retry_policy_for_node(node_type: String) -> RetryPolicy {
    storage::get_retry_policy(&node_type)
        .unwrap_or_default()
}

// Event Processing
async fn process_event(event: &WorkflowEvent) -> Result<Vec<String>, String> {
    let triggered_executions = Vec::new();
    
    let event_listeners = storage::get_event_listeners(&event.event_type);
    let listeners_to_trigger: Vec<EventListener> = event_listeners.iter()
        .filter(|listener| listener.active && matches_conditions(&listener.conditions, &event.data))
        .cloned()
        .collect();
    
    for listener in listeners_to_trigger {
        let workflow_id = listener.workflow_id.clone();
        let event_data = event.data.clone();
        spawn(async move {
            let execution_result = start_execution(workflow_id, Some(event_data)).await;
            match execution_result {
                Ok(execution_id) => {
                }
                Err(e) => {
                }
            }
        });
    }
    
    Ok(triggered_executions)
}

fn matches_conditions(
    conditions: &HashMap<String, ConfigValue>,
    event_data: &HashMap<String, ConfigValue>
) -> bool {
    for (key, expected_value) in conditions {
        if let Some(actual_value) = event_data.get(key) {
            if !values_match(expected_value, actual_value) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

fn values_match(expected: &ConfigValue, actual: &ConfigValue) -> bool {
    match (expected, actual) {
        (ConfigValue::String(e), ConfigValue::String(a)) => e == a,
        (ConfigValue::Number(e), ConfigValue::Number(a)) => (e - a).abs() < f64::EPSILON,
        (ConfigValue::Boolean(e), ConfigValue::Boolean(a)) => e == a,
        _ => false,
    }
}

// Timer and Scheduling
async fn setup_schedule_timer(schedule: &ScheduledWorkflow) -> Result<String, String> {
    let workflow_id = schedule.workflow_id.clone();
    let schedule_id = schedule.id.clone();
    let delay_ns = Duration::from_millis(
        (schedule.next_execution.saturating_sub(api::time())) / 1_000_000
    );
    
    let timer_id = set_timer(delay_ns, move || {
        spawn(async move {
            let execution_result = start_execution(workflow_id, None).await;
            match execution_result {
                Ok(execution_id) => {
                    reschedule_workflow(schedule_id).await;
                }
                Err(e) => {
                }
            }
        });
    });
    
    Ok(format!("{:?}", timer_id))
}

async fn reschedule_workflow(schedule_id: String) {
    if let Some(mut schedule) = storage::get_scheduled_workflow(&schedule_id) {
        if let Ok(next_execution) = calculate_next_execution(&schedule.cron_expression) {
            schedule.next_execution = next_execution;
            
            spawn(async move {
                if let Ok(timer_id) = setup_schedule_timer(&schedule).await {
                    let mut sched = schedule;
                    sched.timer_id = Some(timer_id);
                    storage::insert_scheduled_workflow(schedule_id, sched);
                }
            });
        }
    }
}

fn calculate_next_execution(cron_expression: &str) -> Result<u64, String> {
    let current_time = api::time();
    
    if cron_expression.starts_with("*/") {
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        if let Some(minute_part) = parts.first() {
            if let Some(interval_str) = minute_part.strip_prefix("*/") {
                if let Ok(interval_minutes) = interval_str.parse::<u64>() {
                    let interval_ns = interval_minutes * 60 * 1_000_000_000;
                    return Ok(current_time + interval_ns);
                }
            }
        }
    }
    
    Ok(current_time + 3600 * 1_000_000_000)
}

pub fn restore_scheduled_workflows() {
    let schedule_list: Vec<ScheduledWorkflow> = storage::SCHEDULED_WORKFLOWS.with(|schedules| {
        schedules.borrow().iter()
            .map(|(_, storable)| storable.0.clone())
            .collect()
    });
    
    for schedule in schedule_list {
        if schedule.active {
            let schedule_id = schedule.id.clone();
            spawn(async move {
                if let Ok(timer_id) = setup_schedule_timer(&schedule).await {
                    let mut sched = schedule;
                    sched.timer_id = Some(timer_id);
                    storage::insert_scheduled_workflow(schedule_id, sched);
                }
            });
        }
    }
    
    // Also restore persistent scheduled executions
    restore_persistent_timers();
}

// Enhanced persistent timer system
#[update]
pub async fn schedule_workflow_execution(
    workflow_id: String, 
    delay_seconds: u64,
    schedule_type: ScheduleType
) -> Result<String, String> {
    use std::collections::HashMap;
    
    let current_time = api::time();
    let next_execution = current_time + (delay_seconds * 1_000_000_000); // Convert to nanoseconds
    
    let scheduled_execution = ScheduledExecution {
        workflow_id: workflow_id.clone(),
        next_execution,
        interval: match &schedule_type {
            ScheduleType::Interval { seconds } => Some(*seconds),
            _ => None,
        },
        timer_id: None,
        schedule_type,
        metadata: HashMap::new(),
    };
    
    // Set up persistent timer
    let timer_id = schedule_persistent_timer(&scheduled_execution).await?;
    
    let mut execution = scheduled_execution;
    execution.timer_id = Some(timer_id);
    
    // Store in persistent storage
    storage::insert_scheduled_execution(workflow_id.clone(), execution);
    
    Ok(workflow_id)
}

async fn schedule_persistent_timer(execution: &ScheduledExecution) -> Result<String, String> {
    let workflow_id = execution.workflow_id.clone();
    let schedule_type = execution.schedule_type.clone();
    let current_time = api::time();
    
    // Calculate delay in nanoseconds
    let delay_ns = if execution.next_execution > current_time {
        execution.next_execution - current_time
    } else {
        0 // Execute immediately if overdue
    };
    
    let delay = Duration::from_nanos(delay_ns);
    
    let timer_id = set_timer(delay, move || {
        spawn(async move {
            
            // Execute the workflow
            if let Ok(execution_id) = start_execution(workflow_id.clone(), None).await {
            }
            
            // Reschedule if recurring
            reschedule_persistent_execution(workflow_id, schedule_type).await;
        });
    });
    
    Ok(format!("{:?}", timer_id))
}

async fn reschedule_persistent_execution(workflow_id: String, schedule_type: ScheduleType) {
    match schedule_type {
        ScheduleType::Interval { seconds } => {
            // Reschedule for next interval
            let current_time = api::time();
            let next_execution = current_time + (seconds * 1_000_000_000);
            
            if let Some(mut execution) = storage::get_scheduled_execution(&workflow_id) {
                execution.next_execution = next_execution;
                
                // Schedule new timer
                if let Ok(timer_id) = schedule_persistent_timer(&execution).await {
                    execution.timer_id = Some(timer_id);
                    storage::insert_scheduled_execution(workflow_id, execution);
                }
            }
        }
        ScheduleType::Cron { expression } => {
            // Recalculate next execution based on cron expression
            if let Ok(next_execution) = calculate_next_execution(&expression) {
                if let Some(mut execution) = storage::get_scheduled_execution(&workflow_id) {
                    execution.next_execution = next_execution;
                    
                    if let Ok(timer_id) = schedule_persistent_timer(&execution).await {
                        execution.timer_id = Some(timer_id);
                        storage::insert_scheduled_execution(workflow_id, execution);
                    }
                }
            }
        }
        ScheduleType::Once => {
            // Remove one-time executions
            storage::remove_scheduled_execution(&workflow_id);
        }
        ScheduleType::Heartbeat => {
            // Heartbeat executions are handled by the heartbeat function
            // No rescheduling needed
        }
    }
}

pub fn restore_persistent_timers() {
    let current_time = api::time();
    let all_executions = storage::list_all_scheduled_executions();
    
    
    for (workflow_id, execution) in all_executions {
        if execution.next_execution <= current_time {
            // Execute immediately if overdue
            let wf_id = workflow_id.clone();
            spawn(async move {
                if let Ok(execution_id) = start_execution(wf_id.clone(), None).await {
                }
            });
            
            // Reschedule based on type
            let schedule_type = execution.schedule_type.clone();
            spawn(async move {
                reschedule_persistent_execution(workflow_id, schedule_type).await;
            });
        } else {
            // Reschedule for future execution
            let wf_id = workflow_id.clone();
            spawn(async move {
                if let Ok(timer_id) = schedule_persistent_timer(&execution).await {
                    let mut exec = execution;
                    exec.timer_id = Some(timer_id);
                    storage::insert_scheduled_execution(wf_id, exec);
                }
            });
        }
    }
}

#[query]
pub fn list_persistent_scheduled_executions() -> Vec<(String, ScheduledExecution)> {
    storage::list_all_scheduled_executions()
}

#[update]
pub async fn cancel_persistent_execution(workflow_id: String) -> Result<(), String> {
    if storage::remove_scheduled_execution(&workflow_id).is_some() {
        Ok(())
    } else {
        Err("No persistent execution found for workflow".to_string())
    }
}