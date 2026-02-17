use crate::{AccessEvent, AnomalyFlags};
use chrono::{Timelike, Utc};
use serde::{Deserialize, Serialize};

/// My sovereign baseline for anomaly detection and RoH gating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProfile {
    pub subject_did: String,
    pub roh_ceiling: f32,      // e.g., 0.3
    pub min_roh_floor: f32,    // e.g., 0.05 so nothing is ever declared "zero risk"
    pub trusted_regions: Vec<String>,
    pub typical_hours_local: (u8, u8), // (start_hour, end_hour) in my local time
    pub known_user_agents: Vec<String>,
    pub mfa_required: bool,
    pub allow_some_freedom_sacrifice: bool,
}

impl Default for BaselineProfile {
    fn default() -> Self {
        Self {
            subject_did: "did:bostrom:example".into(),
            roh_ceiling: 0.3,
            min_roh_floor: 0.05,
            trusted_regions: vec!["Phoenix, AZ".into()],
            typical_hours_local: (7, 23),
            known_user_agents: Vec::new(),
            mfa_required: true,
            allow_some_freedom_sacrifice: true,
        }
    }
}

/// Compute anomaly flags using my sovereignty baselines.
pub fn compute_anomaly_flags(
    ev: &AccessEvent,
    baseline: &BaselineProfile,
) -> AnomalyFlags {
    let mut flags = AnomalyFlags::default();

    // Region and impossible travel checks are simplified;
    // in a full implementation, you'd correlate with prior events.
    if let Some(region) = &ev.region {
        if !baseline.trusted_regions.iter().any(|r| region.contains(r)) {
            flags.unusual_region = true;
        }
    }

    // Time of day check – using UTC hour; you can adjust with your local offset.
    let hour = ev.timestamp.with_timezone(&Utc).hour() as u8;
    let (start, end) = baseline.typical_hours_local;
    if hour < start || hour > end {
        flags.unusual_time_of_day = true;
    }

    // New user agent.
    if let Some(ua) = &ev.user_agent {
        if !baseline.known_user_agents.iter().any(|k| ua.contains(k)) {
            flags.new_user_agent = true;
        }
    }

    if baseline.mfa_required && ev.success && ev.mfa_used == Some(false) {
        flags.mfa_missing_when_expected = true;
    }

    // Token reuse requires cross‑event analysis – stubbed here as false.
    flags.token_reuse_suspected = false;

    flags
}

/// Sovereign cybostate gate: if RoH exceeds my ceiling, the system must favor safety over freedom.
pub fn cybostate_gate(roh_score: f32, baseline: &BaselineProfile) -> bool {
    if roh_score <= baseline.roh_ceiling {
        true
    } else if baseline.allow_some_freedom_sacrifice {
        // I explicitly allow sacrificing some freedom for security:
        // anything above the ceiling is treated as NOT OK for evolution or privilege.
        false
    } else {
        // If I ever disable sacrifice, we still mark it, but do not hard‑veto here.
        true
    }
}
