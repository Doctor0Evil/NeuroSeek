//! Governance logging for audio sessions, tagging with nanoswarm telemetry and ResponseMetric.

use crate::audio_nanopolytope::{AudioNanopolytope, AudioState};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Telemetry from nanoswarms (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoswarmTelemetry {
    pub thermal_distance_index: f64,
    pub molecular_balance_index: f64,
    pub fatigue: f64,
    pub risk: f64,
    pub eco_impact: f64,
    pub nicotine_burden: f64, // from lung mapping
}

/// Log entry for one audio session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: String,
    pub timestamp: SystemTime,
    pub polytope_name: String,
    pub audio_state: AudioState,
    pub predicted_biophysical: Option<[f64; 5]>,
    pub telemetry_before: Option<NanoswarmTelemetry>,
    pub telemetry_after: Option<NanoswarmTelemetry>,
    pub response_metric_k: f64,
    pub response_metric_d: f64,
    pub response_metric_dw: f64,
}

impl SessionLog {
    /// Create a new log entry.
    pub fn new(
        polytope: &AudioNanopolytope,
        state: AudioState,
        telemetry_before: Option<NanoswarmTelemetry>,
    ) -> Self {
        let predicted = polytope.map_to_biophysical(&state);
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            polytope_name: polytope.name.clone(),
            audio_state: state,
            predicted_biophysical: predicted,
            telemetry_before,
            telemetry_after: None,
            response_metric_k: polytope.k,
            response_metric_d: polytope.d,
            response_metric_dw: polytope.dw,
        }
    }

    /// Add after-session telemetry.
    pub fn with_after(mut self, telemetry: NanoswarmTelemetry) -> Self {
        self.telemetry_after = Some(telemetry);
        self
    }
}
