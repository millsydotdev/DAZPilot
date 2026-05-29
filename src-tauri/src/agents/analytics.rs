use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct AgentMetrics {
    pub agent_type: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_duration_ms: u64,
    pub last_execution: Option<String>,
    pub last_success: bool,
    pub average_duration_ms: f64,
    // Per-command metrics
    pub command_metrics: HashMap<String, CommandMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandMetrics {
    pub command: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_duration_ms: u64,
    pub last_execution: Option<String>,
    pub last_success: bool,
    pub average_duration_ms: f64,
}

// Multi-turn loop specific metrics
#[derive(Debug, Clone, Serialize)]
pub struct MultiTurnMetrics {
    pub total_loops: u64,
    pub completed_loops: u64,
    pub terminated_early: u64,  // Loops that ended before max turns
    pub max_turns_reached: u64, // Loops that hit the turn limit
    pub average_turns_per_loop: f64,
    pub total_actions_executed: u64,
    pub successful_actions: u64,
    pub failed_actions: u64,
}

static AGENT_STATS: Lazy<Mutex<HashMap<String, AgentMetrics>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static MULTI_TURN_STATS: Lazy<Mutex<MultiTurnMetrics>> = Lazy::new(|| {
    Mutex::new(MultiTurnMetrics {
        total_loops: 0,
        completed_loops: 0,
        terminated_early: 0,
        max_turns_reached: 0,
        average_turns_per_loop: 0.0,
        total_actions_executed: 0,
        successful_actions: 0,
        failed_actions: 0,
    })
});

pub fn record_execution(agent_type: &str, success: bool, duration: std::time::Duration) {
    let mut stats = AGENT_STATS.lock().unwrap();
    let entry = stats.entry(agent_type.to_string()).or_insert(AgentMetrics {
        agent_type: agent_type.to_string(),
        total_executions: 0,
        successful_executions: 0,
        failed_executions: 0,
        total_duration_ms: 0,
        last_execution: None,
        last_success: true,
        average_duration_ms: 0.0,
        command_metrics: HashMap::new(),
    });

    entry.total_executions += 1;
    if success {
        entry.successful_executions += 1;
    } else {
        entry.failed_executions += 1;
    }
    entry.last_success = success;
    entry.last_execution = Some(format!("{:?}", std::time::SystemTime::now()));
    entry.total_duration_ms += duration.as_millis() as u64;
    entry.average_duration_ms = entry.total_duration_ms as f64 / entry.total_executions as f64;
}

pub fn record_command_execution(
    agent_type: &str,
    command: &str,
    success: bool,
    duration: std::time::Duration,
) {
    let mut stats = AGENT_STATS.lock().unwrap();
    let entry = stats.entry(agent_type.to_string()).or_insert(AgentMetrics {
        agent_type: agent_type.to_string(),
        total_executions: 0,
        successful_executions: 0,
        failed_executions: 0,
        total_duration_ms: 0,
        last_execution: None,
        last_success: true,
        average_duration_ms: 0.0,
        command_metrics: HashMap::new(),
    });

    let cmd_entry = entry
        .command_metrics
        .entry(command.to_string())
        .or_insert(CommandMetrics {
            command: command.to_string(),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_duration_ms: 0,
            last_execution: None,
            last_success: true,
            average_duration_ms: 0.0,
        });

    cmd_entry.total_executions += 1;
    if success {
        cmd_entry.successful_executions += 1;
    } else {
        cmd_entry.failed_executions += 1;
    }
    cmd_entry.last_success = success;
    cmd_entry.last_execution = Some(format!("{:?}", std::time::SystemTime::now()));
    cmd_entry.total_duration_ms += duration.as_millis() as u64;
    cmd_entry.average_duration_ms =
        cmd_entry.total_duration_ms as f64 / cmd_entry.total_executions as f64;
}

// Multi-turn loop metrics
pub fn record_multi_turn_start() {
    let mut stats = MULTI_TURN_STATS.lock().unwrap();
    stats.total_loops += 1;
}

pub fn record_multi_turn_complete(turns_used: u64, max_turns: u64, success: bool) {
    let mut stats = MULTI_TURN_STATS.lock().unwrap();
    if success {
        stats.completed_loops += 1;
    }

    if turns_used < max_turns {
        stats.terminated_early += 1;
    } else if turns_used >= max_turns {
        stats.max_turns_reached += 1;
    }

    stats.total_actions_executed += turns_used;
    stats.successful_actions += if success { turns_used } else { 0 };
    stats.failed_actions += if !success { turns_used } else { 0 };

    // Recalculate average turns per loop
    if stats.total_loops > 0 {
        stats.average_turns_per_loop =
            stats.total_actions_executed as f64 / stats.total_loops as f64;
    }
}

pub fn get_agent_metrics() -> Vec<AgentMetrics> {
    let stats = AGENT_STATS.lock().unwrap();
    let mut metrics: Vec<AgentMetrics> = stats.values().cloned().collect();
    metrics.sort_by(|a, b| a.agent_type.cmp(&b.agent_type));
    metrics
}

pub fn get_multi_turn_metrics() -> MultiTurnMetrics {
    let stats = MULTI_TURN_STATS.lock().unwrap();
    stats.clone()
}

pub fn get_metrics() -> Vec<AgentMetrics> {
    let stats = AGENT_STATS.lock().unwrap();
    let mut metrics: Vec<AgentMetrics> = stats.values().cloned().collect();
    metrics.sort_by(|a, b| a.agent_type.cmp(&b.agent_type));
    metrics
}

pub fn start_timer() -> Instant {
    Instant::now()
}
