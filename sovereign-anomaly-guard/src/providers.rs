use crate::{AccessEvent, AnomalyAssessment, AnomalyFlags, AnomalySeverity};
use crate::roh;
use crate::sovereignty;
use chrono::{DateTime, Utc};
use csv::ReaderBuilder;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;
use uuid::Uuid;

/// Example: simplified Google login CSV row (you map this to your real export).
#[derive(Debug, Deserialize)]
struct GoogleLoginRow {
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "IP address")]
    ip: String,
    #[serde(rename = "Location")]
    location: String,
    #[serde(rename = "User Agent")]
    user_agent: String,
    #[serde(rename = "Event")]
    event: String,
}

/// Parse Google CSV into my neutral AccessEvent stream.
pub fn ingest_google_csv<P: AsRef<Path>>(
    path: P,
    account_id: &str,
    subject_did: &str,
) -> anyhow::Result<Vec<AccessEvent>> {
    let path_ref = path.as_ref();
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path_ref)?;

    let mut events = Vec::new();
    for result in rdr.deserialize::<GoogleLoginRow>() {
        let row = result?;
        let timestamp: DateTime<Utc> = row.time.parse()?; // ensure ISO‑8601 in export
        let event = AccessEvent {
            id: Uuid::new_v4(),
            provider: "google".into(),
            account_id: account_id.to_string(),
            timestamp,
            ip: row.ip,
            region: Some(row.location),
            user_agent: Some(row.user_agent),
            mfa_used: None, // fill if available in export
            success: row.event.contains("Successful"),
            raw_source_path: path_ref.to_string_lossy().into_owned(),
        };
        events.push(event);
    }

    Ok(events)
}

/// Flag anomalies for a batch of events using sovereign baselines.
pub fn assess_events_with_sovereign_guards(
    subject_did: &str,
    provider: &str,
    events: &[crate::AccessEvent],
    baseline: &sovereignty::BaselineProfile,
) -> Vec<AnomalyAssessment> {
    events
        .iter()
        .map(|ev| {
            let flags = sovereignty::compute_anomaly_flags(ev, baseline);
            let roh_score = roh::estimate_roh(ev, &flags, baseline);
            let cybostate_ok = sovereignty::cybostate_gate(roh_score, baseline);

            let severity = if !ev.success {
                AnomalySeverity::Info
            } else if roh_score >= 0.8 {
                AnomalySeverity::Critical
            } else if roh_score >= baseline.roh_ceiling {
                AnomalySeverity::High
            } else if flags.impossible_travel
                || flags.unusual_region
                || flags.token_reuse_suspected
            {
                AnomalySeverity::Medium
            } else if flags.new_user_agent || flags.unusual_time_of_day {
                AnomalySeverity::Low
            } else {
                AnomalySeverity::Info
            };

            AnomalyAssessment {
                event_id: ev.id,
                severity,
                flags,
                roh_score,
                cybostate_ok,
            }
        })
        .collect()
}
