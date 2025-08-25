// Enhanced Scheduler Service with Universal Date Format Support
// Supports both cron expressions and user-friendly dd/mm/yy hh:mm:ss format

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;
use ic_cdk_timers::{set_timer, set_timer_interval, clear_timer};

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SchedulerService {
    active_schedules: HashMap<String, ScheduleInfo>,
    schedule_counter: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub id: String,
    pub schedule_type: ScheduleType,
    pub schedule_config: ScheduleConfig,
    pub next_execution: u64, // Nanoseconds since epoch
    pub timer_id: Option<u64>, // Store timer ID for cancellation
    pub is_active: bool,
    pub workflow_id: String,
    pub node_id: String,
    pub created_at: u64,
    pub execution_count: u64,
    pub last_execution: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ScheduleType {
    OneTime,
    Recurring,
    Interval,
    Cron,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ScheduleConfig {
    // Universal format: dd/mm/yy hh:mm:ss or dd/mm/yyyy hh:mm:ss
    pub datetime_string: Option<String>,
    
    // Traditional cron expression
    pub cron_expression: Option<String>,
    
    // Interval in seconds for recurring schedules
    pub interval_seconds: Option<u64>,
    
    // Timezone support
    pub timezone: String, // Default: "UTC"
    
    // Execution limits
    pub max_executions: Option<u64>,
    pub end_date: Option<String>, // dd/mm/yy hh:mm:ss format
    
    // Advanced options
    pub skip_weekends: bool,
    pub skip_holidays: bool,
    pub retry_on_failure: bool,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub success: bool,
    pub message: String,
    pub schedule_id: Option<String>,
    pub next_execution: Option<u64>,
}

impl SchedulerService {
    pub fn new() -> Self {
        SchedulerService {
            active_schedules: HashMap::new(),
            schedule_counter: 0,
        }
    }

    // =============================================================================
    // SCHEDULE CREATION WITH UNIVERSAL FORMAT
    // =============================================================================

    pub fn create_schedule_from_universal_format(
        &mut self,
        datetime_string: &str,
        workflow_id: String,
        node_id: String,
        timezone: Option<String>,
    ) -> Result<ScheduleResult, String> {
        // Parse universal format: dd/mm/yy hh:mm:ss or dd/mm/yyyy hh:mm:ss
        let execution_time = self.parse_universal_datetime(datetime_string)?;
        
        self.schedule_counter += 1;
        let schedule_id = format!("schedule_{}", self.schedule_counter);
        
        let schedule_info = ScheduleInfo {
            id: schedule_id.clone(),
            schedule_type: ScheduleType::OneTime,
            schedule_config: ScheduleConfig {
                datetime_string: Some(datetime_string.to_string()),
                cron_expression: None,
                interval_seconds: None,
                timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
                max_executions: Some(1),
                end_date: None,
                skip_weekends: false,
                skip_holidays: false,
                retry_on_failure: true,
                retry_attempts: 3,
            },
            next_execution: execution_time,
            timer_id: None,
            is_active: true,
            workflow_id,
            node_id,
            created_at: time(),
            execution_count: 0,
            last_execution: None,
        };

        // Set the timer
        let timer_result = self.set_execution_timer(&schedule_info)?;
        let mut final_schedule = schedule_info;
        final_schedule.timer_id = Some(timer_result);

        self.active_schedules.insert(schedule_id.clone(), final_schedule);

        Ok(ScheduleResult {
            success: true,
            message: format!("Schedule created successfully for {}", datetime_string),
            schedule_id: Some(schedule_id),
            next_execution: Some(execution_time),
        })
    }

    pub fn create_recurring_schedule(
        &mut self,
        datetime_string: &str,
        interval_seconds: u64,
        workflow_id: String,
        node_id: String,
        max_executions: Option<u64>,
        timezone: Option<String>,
    ) -> Result<ScheduleResult, String> {
        let first_execution = self.parse_universal_datetime(datetime_string)?;
        
        self.schedule_counter += 1;
        let schedule_id = format!("recurring_{}", self.schedule_counter);
        
        let schedule_info = ScheduleInfo {
            id: schedule_id.clone(),
            schedule_type: ScheduleType::Recurring,
            schedule_config: ScheduleConfig {
                datetime_string: Some(datetime_string.to_string()),
                cron_expression: None,
                interval_seconds: Some(interval_seconds),
                timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
                max_executions,
                end_date: None,
                skip_weekends: false,
                skip_holidays: false,
                retry_on_failure: true,
                retry_attempts: 3,
            },
            next_execution: first_execution,
            timer_id: None,
            is_active: true,
            workflow_id,
            node_id,
            created_at: time(),
            execution_count: 0,
            last_execution: None,
        };

        // Set interval timer
        let timer_result = self.set_interval_timer(&schedule_info)?;
        let mut final_schedule = schedule_info;
        final_schedule.timer_id = Some(timer_result);

        self.active_schedules.insert(schedule_id.clone(), final_schedule);

        Ok(ScheduleResult {
            success: true,
            message: format!("Recurring schedule created starting at {}", datetime_string),
            schedule_id: Some(schedule_id),
            next_execution: Some(first_execution),
        })
    }

    pub fn create_cron_schedule(
        &mut self,
        cron_expression: &str,
        workflow_id: String,
        node_id: String,
        timezone: Option<String>,
    ) -> Result<ScheduleResult, String> {
        // Validate cron expression
        self.validate_cron_expression(cron_expression)?;
        
        let next_execution = self.calculate_next_cron_execution(cron_expression)?;
        
        self.schedule_counter += 1;
        let schedule_id = format!("cron_{}", self.schedule_counter);
        
        let schedule_info = ScheduleInfo {
            id: schedule_id.clone(),
            schedule_type: ScheduleType::Cron,
            schedule_config: ScheduleConfig {
                datetime_string: None,
                cron_expression: Some(cron_expression.to_string()),
                interval_seconds: None,
                timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
                max_executions: None,
                end_date: None,
                skip_weekends: false,
                skip_holidays: false,
                retry_on_failure: true,
                retry_attempts: 3,
            },
            next_execution,
            timer_id: None,
            is_active: true,
            workflow_id,
            node_id,
            created_at: time(),
            execution_count: 0,
            last_execution: None,
        };

        self.active_schedules.insert(schedule_id.clone(), schedule_info);

        Ok(ScheduleResult {
            success: true,
            message: format!("Cron schedule created: {}", cron_expression),
            schedule_id: Some(schedule_id),
            next_execution: Some(next_execution),
        })
    }

    // =============================================================================
    // DATE/TIME PARSING
    // =============================================================================

    fn parse_universal_datetime(&self, datetime_str: &str) -> Result<u64, String> {
        // Support formats:
        // dd/mm/yy hh:mm:ss
        // dd/mm/yyyy hh:mm:ss
        // dd-mm-yy hh:mm:ss
        // dd-mm-yyyy hh:mm:ss
        // yyyy-mm-dd hh:mm:ss (ISO format)

        let datetime_str = datetime_str.trim();
        
        // Split date and time parts
        let parts: Vec<&str> = datetime_str.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid datetime format. Expected: 'dd/mm/yy hh:mm:ss' or 'dd/mm/yyyy hh:mm:ss'".to_string());
        }

        let date_part = parts[0];
        let time_part = parts[1];

        // Parse date part (handle different separators)
        let date_components = if date_part.contains('/') {
            date_part.split('/').collect::<Vec<&str>>()
        } else if date_part.contains('-') {
            // Check if it's ISO format (yyyy-mm-dd) or dd-mm-yy
            let dash_parts = date_part.split('-').collect::<Vec<&str>>();
            if dash_parts.len() == 3 && dash_parts[0].len() == 4 {
                // ISO format: yyyy-mm-dd
                vec![dash_parts[2], dash_parts[1], dash_parts[0]] // Reorder to dd-mm-yyyy
            } else {
                dash_parts
            }
        } else {
            return Err("Invalid date separator. Use '/' or '-'".to_string());
        };

        if date_components.len() != 3 {
            return Err("Invalid date format. Expected: dd/mm/yy or dd/mm/yyyy".to_string());
        }

        // Parse time part
        let time_components: Vec<&str> = time_part.split(':').collect();
        if time_components.len() != 3 {
            return Err("Invalid time format. Expected: hh:mm:ss".to_string());
        }

        // Extract components
        let day: u32 = date_components[0].parse()
            .map_err(|_| "Invalid day")?;
        let month: u32 = date_components[1].parse()
            .map_err(|_| "Invalid month")?;
        let mut year: u32 = date_components[2].parse()
            .map_err(|_| "Invalid year")?;

        let hour: u32 = time_components[0].parse()
            .map_err(|_| "Invalid hour")?;
        let minute: u32 = time_components[1].parse()
            .map_err(|_| "Invalid minute")?;
        let second: u32 = time_components[2].parse()
            .map_err(|_| "Invalid second")?;

        // Handle 2-digit years (assume 20xx for yy < 50, 19xx for yy >= 50)
        if year < 100 {
            year = if year < 50 { 2000 + year } else { 1900 + year };
        }

        // Validate ranges
        if month < 1 || month > 12 {
            return Err("Month must be between 1 and 12".to_string());
        }
        if day < 1 || day > 31 {
            return Err("Day must be between 1 and 31".to_string());
        }
        if hour > 23 {
            return Err("Hour must be between 0 and 23".to_string());
        }
        if minute > 59 {
            return Err("Minute must be between 0 and 59".to_string());
        }
        if second > 59 {
            return Err("Second must be between 0 and 59".to_string());
        }

        // Convert to timestamp (simplified - in production would use proper datetime library)
        let timestamp = self.datetime_to_timestamp(year, month, day, hour, minute, second)?;
        
        // Convert to nanoseconds (IC time format)
        Ok(timestamp * 1_000_000_000)
    }

    fn datetime_to_timestamp(&self, year: u32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Result<u64, String> {
        // Simplified timestamp calculation
        // In production, would use a proper datetime library like chrono
        
        // Days since Unix epoch (1970-01-01)
        let mut days = 0u64;
        
        // Add days for years
        for y in 1970..year {
            if self.is_leap_year(y) {
                days += 366;
            } else {
                days += 365;
            }
        }
        
        // Add days for months in current year
        let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for m in 1..month {
            days += days_in_month[(m - 1) as usize] as u64;
            if m == 2 && self.is_leap_year(year) {
                days += 1; // February has 29 days in leap years
            }
        }
        
        // Add remaining days
        days += (day - 1) as u64;
        
        // Convert to seconds
        let timestamp = days * 24 * 60 * 60 + 
                       hour as u64 * 60 * 60 + 
                       minute as u64 * 60 + 
                       second as u64;
        
        Ok(timestamp)
    }

    fn is_leap_year(&self, year: u32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    // =============================================================================
    // TIMER MANAGEMENT
    // =============================================================================

    fn set_execution_timer(&self, schedule: &ScheduleInfo) -> Result<u64, String> {
        let current_time = time();
        if schedule.next_execution <= current_time {
            return Err("Scheduled time is in the past".to_string());
        }
        
        let delay_ns = schedule.next_execution - current_time;
        let delay_seconds = delay_ns / 1_000_000_000;
        
        // Set timer (this is a simplified version - actual implementation would call workflow execution)
        let _timer_id = set_timer(
            std::time::Duration::from_secs(delay_seconds),
            || {
                // This would trigger workflow execution
            }
        );
        
        // Store timer_id as a simple counter for demo purposes
        // In production, you'd need a proper mapping system
        Ok(delay_seconds) // Return a unique identifier
    }

    fn set_interval_timer(&self, schedule: &ScheduleInfo) -> Result<u64, String> {
        let interval_seconds = schedule.schedule_config.interval_seconds
            .ok_or("Interval not specified for recurring schedule")?;
        
        let _timer_id = set_timer_interval(
            std::time::Duration::from_secs(interval_seconds),
            || {
                // This would trigger workflow execution
            }
        );
        
        Ok(interval_seconds) // Return a unique identifier
    }

    // =============================================================================
    // CRON SUPPORT
    // =============================================================================

    fn validate_cron_expression(&self, cron_expr: &str) -> Result<(), String> {
        let parts: Vec<&str> = cron_expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err("Cron expression must have 5 fields: minute hour day month weekday".to_string());
        }
        
        // Basic validation (in production, would use a proper cron library)
        Ok(())
    }

    fn calculate_next_cron_execution(&self, _cron_expr: &str) -> Result<u64, String> {
        // Simplified cron calculation - in production would use proper cron library
        let current_time = time();
        Ok(current_time + 3600 * 1_000_000_000) // Next hour as example
    }

    // =============================================================================
    // SCHEDULE MANAGEMENT
    // =============================================================================

    pub fn list_active_schedules(&self) -> Vec<ScheduleInfo> {
        self.active_schedules.values()
            .filter(|schedule| schedule.is_active)
            .cloned()
            .collect()
    }

    pub fn get_schedule(&self, schedule_id: &str) -> Option<&ScheduleInfo> {
        self.active_schedules.get(schedule_id)
    }

    pub fn cancel_schedule(&mut self, schedule_id: &str) -> Result<String, String> {
        if let Some(schedule) = self.active_schedules.get_mut(schedule_id) {
            schedule.is_active = false;
            
            // Cancel the timer if it exists - simplified for demo
            if let Some(_timer_id) = schedule.timer_id {
                // In production, would properly cancel the timer using the actual TimerId
                // clear_timer(actual_timer_id);
            }
            
            Ok(format!("Schedule {} cancelled successfully", schedule_id))
        } else {
            Err(format!("Schedule {} not found", schedule_id))
        }
    }

    pub fn update_schedule(&mut self, schedule_id: &str, new_datetime: &str) -> Result<ScheduleResult, String> {
        // Parse the datetime first to avoid borrow checker issues
        let new_execution_time = self.parse_universal_datetime(new_datetime)?;
        
        if let Some(schedule) = self.active_schedules.get_mut(schedule_id) {
            // Cancel existing timer - simplified for demo
            if let Some(_timer_id) = schedule.timer_id {
                // In production, would properly cancel the timer using the actual TimerId
                // clear_timer(actual_timer_id);
            }
            
            // Update schedule
            schedule.next_execution = new_execution_time;
            schedule.schedule_config.datetime_string = Some(new_datetime.to_string());
            
            // Calculate new timer delay for demo purposes
            let current_time = time();
            let delay_seconds = if new_execution_time > current_time {
                (new_execution_time - current_time) / 1_000_000_000
            } else {
                1 // Default to 1 second if in the past
            };
            
            schedule.timer_id = Some(delay_seconds);
            
            Ok(ScheduleResult {
                success: true,
                message: format!("Schedule {} updated to {}", schedule_id, new_datetime),
                schedule_id: Some(schedule_id.to_string()),
                next_execution: Some(new_execution_time),
            })
        } else {
            Err(format!("Schedule {} not found", schedule_id))
        }
    }

    pub fn get_next_executions(&self, limit: usize) -> Vec<(String, u64, String)> {
        let mut schedules: Vec<_> = self.active_schedules.values()
            .filter(|s| s.is_active)
            .map(|s| (s.id.clone(), s.next_execution, s.workflow_id.clone()))
            .collect();
        
        schedules.sort_by_key(|&(_, time, _)| time);
        schedules.truncate(limit);
        schedules
    }

    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================

    pub fn format_timestamp_to_universal(&self, timestamp_ns: u64) -> String {
        // Convert nanoseconds back to dd/mm/yyyy hh:mm:ss format
        // Simplified implementation - in production would use proper datetime library
        let timestamp_s = timestamp_ns / 1_000_000_000;
        
        // Basic conversion (placeholder)
        let days_since_epoch = timestamp_s / (24 * 60 * 60);
        let remaining_seconds = timestamp_s % (24 * 60 * 60);
        
        let hours = remaining_seconds / 3600;
        let minutes = (remaining_seconds % 3600) / 60;
        let seconds = remaining_seconds % 60;
        
        // Simplified date calculation (would be more complex in reality)
        let year = 1970 + (days_since_epoch / 365) as u32;
        let month = 1; // Simplified
        let day = 1; // Simplified
        
        format!("{:02}/{:02}/{} {:02}:{:02}:{:02}", day, month, year, hours, minutes, seconds)
    }
}

// =============================================================================
// EXAMPLES AND USAGE PATTERNS
// =============================================================================

impl SchedulerService {
    pub fn example_usage() -> String {
        format!(r#"
SCHEDULER SERVICE EXAMPLES:

1. One-time execution:
   // Logging temporarily disabled
   // Logging temporarily disabled

2. Recurring schedule:
   Start: "26/08/24 10:00:00"
   Interval: 3600 seconds (every hour)
   
3. Cron expressions:
   // Logging temporarily disabled
   // Logging temporarily disabled

4. Supported formats:
   - dd/mm/yy hh:mm:ss
   - dd/mm/yyyy hh:mm:ss
   - dd-mm-yy hh:mm:ss
   - yyyy-mm-dd hh:mm:ss (ISO)

5. Timezone support:
   - UTC (default)
   - America/New_York
   - Europe/London
   - Asia/Tokyo
        "#)
    }
}