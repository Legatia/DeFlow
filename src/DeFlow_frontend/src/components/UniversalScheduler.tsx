import React, { useState, useEffect } from 'react';
import { Calendar, Clock, Play, Pause, Settings, Info } from 'lucide-react';

interface ScheduleInfo {
  id: string;
  schedule_type: 'OneTime' | 'Recurring' | 'Interval' | 'Cron';
  next_execution: number;
  is_active: boolean;
  workflow_id: string;
  node_id: string;
  created_at: number;
  execution_count: number;
  last_execution?: number;
}

interface ScheduleResult {
  success: boolean;
  message: string;
  schedule_id?: string;
  next_execution?: number;
}

export const UniversalScheduler: React.FC = () => {
  const [scheduleMode, setScheduleMode] = useState<'one_time' | 'recurring' | 'cron'>('one_time');
  const [datetimeFormat, setDatetimeFormat] = useState<'universal' | 'iso' | 'cron'>('universal');
  const [datetimeString, setDatetimeString] = useState('');
  const [cronExpression, setCronExpression] = useState('');
  const [recurringInterval, setRecurringInterval] = useState('3600');
  const [customInterval, setCustomInterval] = useState('');
  const [timezone, setTimezone] = useState('UTC');
  const [maxExecutions, setMaxExecutions] = useState('');
  const [endDate, setEndDate] = useState('');
  const [skipWeekends, setSkipWeekends] = useState(false);
  const [skipHolidays, setSkipHolidays] = useState(false);
  const [retryOnFailure, setRetryOnFailure] = useState(true);
  const [retryAttempts, setRetryAttempts] = useState(3);
  
  const [activeSchedules, setActiveSchedules] = useState<ScheduleInfo[]>([]);
  const [createResult, setCreateResult] = useState<ScheduleResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [showExamples, setShowExamples] = useState(false);

  // Sample workflow/node IDs for demo
  const [workflowId] = useState('demo_workflow_001');
  const [nodeId] = useState('demo_scheduler_node');

  useEffect(() => {
    loadActiveSchedules();
  }, []);

  const loadActiveSchedules = async () => {
    try {
      // In a real implementation, this would call the IC canister
      // const schedules = await actor.list_schedules();
      // setActiveSchedules(schedules);
      
      // For demo purposes, using mock data
      setActiveSchedules([
        {
          id: 'schedule_1',
          schedule_type: 'OneTime',
          next_execution: Date.now() * 1000000 + 3600000000000, // 1 hour from now
          is_active: true,
          workflow_id: 'workflow_1',
          node_id: 'node_1',
          created_at: Date.now() * 1000000,
          execution_count: 0,
        }
      ]);
    } catch (error) {
      console.error('Failed to load schedules:', error);
    }
  };

  const createSchedule = async () => {
    setIsLoading(true);
    setCreateResult(null);

    try {
      let result: ScheduleResult;

      if (scheduleMode === 'one_time') {
        // result = await actor.create_schedule(
        //   datetimeString,
        //   workflowId,
        //   nodeId,
        //   timezone === 'UTC' ? null : timezone
        // );
        
        // Mock result for demo
        result = {
          success: true,
          message: `Schedule created successfully for ${datetimeString}`,
          schedule_id: `schedule_${Date.now()}`,
          next_execution: Date.now() * 1000000
        };
      } else if (scheduleMode === 'recurring') {
        const intervalSeconds = recurringInterval === 'custom' 
          ? parseInt(customInterval) 
          : parseInt(recurringInterval);
        
        // result = await actor.create_recurring_schedule(
        //   datetimeString,
        //   intervalSeconds,
        //   workflowId,
        //   nodeId,
        //   maxExecutions ? parseInt(maxExecutions) : null,
        //   timezone === 'UTC' ? null : timezone
        // );
        
        // Mock result for demo
        result = {
          success: true,
          message: `Recurring schedule created starting at ${datetimeString}`,
          schedule_id: `recurring_${Date.now()}`,
          next_execution: Date.now() * 1000000
        };
      } else {
        // result = await actor.create_cron_schedule(
        //   cronExpression,
        //   workflowId,
        //   nodeId,
        //   timezone === 'UTC' ? null : timezone
        // );
        
        // Mock result for demo
        result = {
          success: true,
          message: `Cron schedule created: ${cronExpression}`,
          schedule_id: `cron_${Date.now()}`,
          next_execution: Date.now() * 1000000
        };
      }

      setCreateResult(result);
      if (result.success) {
        loadActiveSchedules();
        resetForm();
      }
    } catch (error) {
      setCreateResult({
        success: false,
        message: `Failed to create schedule: ${error}`
      });
    } finally {
      setIsLoading(false);
    }
  };

  const cancelSchedule = async (scheduleId: string) => {
    try {
      // await actor.cancel_schedule(scheduleId);
      loadActiveSchedules();
    } catch (error) {
      console.error('Failed to cancel schedule:', error);
    }
  };

  const resetForm = () => {
    setDatetimeString('');
    setCronExpression('');
    setMaxExecutions('');
    setEndDate('');
  };

  const formatTimestamp = (timestampNs: number): string => {
    const date = new Date(timestampNs / 1000000);
    return date.toLocaleString();
  };

  const getExampleDateTime = (format: string): string => {
    const now = new Date();
    const future = new Date(now.getTime() + 3600000); // 1 hour from now
    
    switch (format) {
      case 'universal':
        return `${future.getDate().toString().padStart(2, '0')}/${(future.getMonth() + 1).toString().padStart(2, '0')}/${future.getFullYear().toString().slice(-2)} ${future.getHours().toString().padStart(2, '0')}:${future.getMinutes().toString().padStart(2, '0')}:00`;
      case 'iso':
        return `${future.getFullYear()}-${(future.getMonth() + 1).toString().padStart(2, '0')}-${future.getDate().toString().padStart(2, '0')} ${future.getHours().toString().padStart(2, '0')}:${future.getMinutes().toString().padStart(2, '0')}:00`;
      default:
        return '';
    }
  };

  const examples = {
    universal: [
      '25/12/24 09:30:00',  // Christmas morning
      '01/01/25 00:00:00',  // New Year midnight
      '15/08/24 14:30:00',  // Afternoon meeting
    ],
    iso: [
      '2024-12-25 09:30:00',
      '2025-01-01 00:00:00',
      '2024-08-15 14:30:00',
    ],
    cron: [
      '0 9 * * 1-5',      // Weekdays at 9 AM
      '0 0 1 * *',        // First day of each month
      '*/15 * * * *',     // Every 15 minutes
    ]
  };

  return (
    <div className="max-w-4xl mx-auto p-6 bg-white rounded-lg shadow-lg">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-2 flex items-center">
          <Calendar className="mr-2" />
          Universal Scheduler
        </h2>
        <p className="text-gray-600">
          Schedule workflows with user-friendly date/time formats. Supports dd/mm/yy hh:mm:ss, ISO format, and cron expressions.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Schedule Creation Form */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-700">Create New Schedule</h3>
          
          {/* Schedule Mode */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Schedule Mode
            </label>
            <select
              value={scheduleMode}
              onChange={(e) => setScheduleMode(e.target.value as any)}
              className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="one_time">One-time Execution</option>
              <option value="recurring">Recurring Schedule</option>
              <option value="cron">Cron Expression</option>
            </select>
          </div>

          {/* Date/Time Format */}
          {scheduleMode !== 'cron' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Date/Time Format
              </label>
              <select
                value={datetimeFormat}
                onChange={(e) => setDatetimeFormat(e.target.value as any)}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="universal">Universal: dd/mm/yy hh:mm:ss</option>
                <option value="iso">ISO: yyyy-mm-dd hh:mm:ss</option>
              </select>
            </div>
          )}

          {/* Date/Time Input */}
          {scheduleMode !== 'cron' ? (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Date and Time
              </label>
              <input
                type="text"
                value={datetimeString}
                onChange={(e) => setDatetimeString(e.target.value)}
                placeholder={getExampleDateTime(datetimeFormat)}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <button
                type="button"
                onClick={() => setDatetimeString(getExampleDateTime(datetimeFormat))}
                className="mt-1 text-xs text-blue-600 hover:text-blue-800"
              >
                Fill with example (1 hour from now)
              </button>
            </div>
          ) : (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Cron Expression
              </label>
              <input
                type="text"
                value={cronExpression}
                onChange={(e) => setCronExpression(e.target.value)}
                placeholder="0 9 * * 1-5"
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          )}

          {/* Recurring Interval */}
          {scheduleMode === 'recurring' && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Recurring Interval
              </label>
              <select
                value={recurringInterval}
                onChange={(e) => setRecurringInterval(e.target.value)}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="300">Every 5 minutes</option>
                <option value="900">Every 15 minutes</option>
                <option value="1800">Every 30 minutes</option>
                <option value="3600">Every hour</option>
                <option value="14400">Every 4 hours</option>
                <option value="43200">Every 12 hours</option>
                <option value="86400">Daily</option>
                <option value="604800">Weekly</option>
                <option value="custom">Custom (seconds)</option>
              </select>
              
              {recurringInterval === 'custom' && (
                <input
                  type="number"
                  value={customInterval}
                  onChange={(e) => setCustomInterval(e.target.value)}
                  placeholder="7200"
                  min="60"
                  max="2592000"
                  className="w-full mt-2 p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              )}
            </div>
          )}

          {/* Timezone */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Timezone
            </label>
            <select
              value={timezone}
              onChange={(e) => setTimezone(e.target.value)}
              className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="UTC">UTC (Coordinated Universal Time)</option>
              <option value="America/New_York">EST - Eastern Time (US)</option>
              <option value="America/Los_Angeles">PST - Pacific Time (US)</option>
              <option value="Europe/London">GMT - Greenwich Mean Time</option>
              <option value="Europe/Paris">CET - Central European Time</option>
              <option value="Asia/Tokyo">JST - Japan Standard Time</option>
              <option value="Asia/Shanghai">CST - China Standard Time</option>
              <option value="Asia/Kolkata">IST - India Standard Time</option>
              <option value="Australia/Sydney">AEST - Australian Eastern Time</option>
            </select>
          </div>

          {/* Advanced Options */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium text-gray-700">Advanced Options</h4>
            
            {scheduleMode === 'recurring' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Max Executions
                </label>
                <input
                  type="number"
                  value={maxExecutions}
                  onChange={(e) => setMaxExecutions(e.target.value)}
                  placeholder="Leave empty for unlimited"
                  min="1"
                  max="10000"
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            )}

            <div className="flex items-center space-x-4">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={skipWeekends}
                  onChange={(e) => setSkipWeekends(e.target.checked)}
                  className="mr-2"
                />
                Skip Weekends
              </label>
              
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={retryOnFailure}
                  onChange={(e) => setRetryOnFailure(e.target.checked)}
                  className="mr-2"
                />
                Retry on Failure
              </label>
            </div>
          </div>

          {/* Create Button */}
          <button
            onClick={createSchedule}
            disabled={isLoading || (!datetimeString && !cronExpression)}
            className="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed flex items-center justify-center"
          >
            {isLoading ? (
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            ) : (
              <>
                <Play className="mr-2 h-4 w-4" />
                Create Schedule
              </>
            )}
          </button>

          {/* Result Message */}
          {createResult && (
            <div className={`p-3 rounded-md ${createResult.success ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
              {createResult.message}
            </div>
          )}
        </div>

        {/* Active Schedules & Examples */}
        <div className="space-y-6">
          {/* Active Schedules */}
          <div>
            <h3 className="text-lg font-semibold text-gray-700 mb-3">Active Schedules</h3>
            
            {activeSchedules.length === 0 ? (
              <p className="text-gray-500 text-center py-4">No active schedules</p>
            ) : (
              <div className="space-y-2">
                {activeSchedules.map((schedule) => (
                  <div key={schedule.id} className="border border-gray-200 rounded-md p-3">
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="flex items-center mb-1">
                          <Clock className="h-4 w-4 mr-1 text-gray-500" />
                          <span className="font-medium text-sm">{schedule.id}</span>
                          <span className={`ml-2 px-2 py-1 text-xs rounded-full ${
                            schedule.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                          }`}>
                            {schedule.is_active ? 'Active' : 'Inactive'}
                          </span>
                        </div>
                        <p className="text-xs text-gray-600">
                          Next: {formatTimestamp(schedule.next_execution)}
                        </p>
                        <p className="text-xs text-gray-600">
                          Executions: {schedule.execution_count}
                        </p>
                      </div>
                      <button
                        onClick={() => cancelSchedule(schedule.id)}
                        className="text-red-600 hover:text-red-800"
                      >
                        <Pause className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Examples */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-lg font-semibold text-gray-700">Examples</h3>
              <button
                onClick={() => setShowExamples(!showExamples)}
                className="text-blue-600 hover:text-blue-800"
              >
                <Info className="h-4 w-4" />
              </button>
            </div>
            
            {showExamples && (
              <div className="space-y-4 text-sm">
                <div>
                  <h4 className="font-medium text-gray-700">Universal Format (dd/mm/yy hh:mm:ss)</h4>
                  <ul className="mt-1 space-y-1 text-gray-600">
                    {examples.universal.map((example, i) => (
                      <li key={i} className="font-mono cursor-pointer hover:bg-gray-100 p-1 rounded"
                          onClick={() => {
                            setDatetimeFormat('universal');
                            setDatetimeString(example);
                          }}>
                        {example}
                      </li>
                    ))}
                  </ul>
                </div>
                
                <div>
                  <h4 className="font-medium text-gray-700">ISO Format (yyyy-mm-dd hh:mm:ss)</h4>
                  <ul className="mt-1 space-y-1 text-gray-600">
                    {examples.iso.map((example, i) => (
                      <li key={i} className="font-mono cursor-pointer hover:bg-gray-100 p-1 rounded"
                          onClick={() => {
                            setDatetimeFormat('iso');
                            setDatetimeString(example);
                          }}>
                        {example}
                      </li>
                    ))}
                  </ul>
                </div>
                
                <div>
                  <h4 className="font-medium text-gray-700">Cron Expressions</h4>
                  <ul className="mt-1 space-y-1 text-gray-600">
                    {examples.cron.map((example, i) => (
                      <li key={i} className="font-mono cursor-pointer hover:bg-gray-100 p-1 rounded"
                          onClick={() => {
                            setScheduleMode('cron');
                            setCronExpression(example);
                          }}>
                        {example}
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default UniversalScheduler;