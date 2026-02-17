//! Governance logging: session logs with telemetry.

use neuroseek_audio::governance::SessionLog as AudioSessionLog;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Telemetry from nanoswarms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoswarmTelemetry {
    pub thermal_distance_index: f64,
    pub molecular_balance_index: f64,
    pub fatigue: f64,
    pub risk: f64,
    pub eco_impact: f64,
    pub nicotine_burden: f64,
}

/// Combined session log for any stimulus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: String,
    pub timestamp: SystemTime,
    pub stimulus_name: String,
    pub audio_log: AudioSessionLog,
    pub telemetry_before: Option<NanoswarmTelemetry>,
    pub telemetry_after: Option<NanoswarmTelemetry>,
    // Could add K/D/DW per session if computed.
}
