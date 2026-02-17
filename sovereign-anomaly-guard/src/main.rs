use chrono::Utc;
use sovereign_anomaly_guard::{
    providers,
    sovereignty::{BaselineProfile},
    AnalysisBatch, BiosessionShard, DonutDecision, DonutloopEntry,
};
use std::fs;

fn main() -> anyhow::Result<()> {
    // 1. Load my baseline (could be read from .rohmodel.aln / .neurorights.json).
    let baseline = BaselineProfile::default();

    // 2. Ingest one provider CSV (example: Google).
    let path = "data/google_logins.csv";
    let subject_did = baseline.subject_did.clone();
    let account_id = "local-google-account-1";
    let events = providers::ingest_google_csv(path, account_id, &subject_did)?;

    // 3. Assess anomalies.
    let assessments =
        providers::assess_events_with_sovereign_guards(&subject_did, "google", &events, &baseline);

    // 4. Derive biosession shard metrics.
    let max_roh = assessments
        .iter()
        .map(|a| a.roh_score)
        .fold(0.0_f32, |acc, x| acc.max(x));
    let cybostate_min = assessments
        .iter()
        .map(|a| if a.cybostate_ok { 1.0 } else { 0.0 })
        .fold(1.0_f32, |acc, x| acc.min(x));
    let anomaly_count = assessments
        .iter()
        .filter(|a| a.severity != sovereign_anomaly_guard::AnomalySeverity::Info)
        .count() as u32;

    let window_start = events
        .first()
        .map(|e| e.timestamp)
        .unwrap_or_else(|| Utc::now());
    let window_end = events
        .last()
        .map(|e| e.timestamp)
        .unwrap_or_else(|| Utc::now());

    // Hash the raw CSV for auditability.
    let raw_bytes = fs::read(path)?;
    let source_hash = sovereign_anomaly_guard::sha256_hex(&raw_bytes);

    let biosession_shard = BiosessionShard {
        subject_did: subject_did.clone(),
        window_start,
        window_end,
        provider: "google".into(),
        total_events: events.len() as u32,
        anomaly_count,
        max_roh,
        cybostate_min,
        roh_ceiling: baseline.roh_ceiling,
        source_hash,
    };

    // 5. Decide mitigation vs monitor based on RoH and my sacrifice flag.
    let decision = if max_roh > baseline.roh_ceiling {
        DonutDecision::Mitigate
    } else if anomaly_count > 0 {
        DonutDecision::Monitor
    } else {
        DonutDecision::Ignore
    };

    // 6. Build donutloop entry for my personal sovereignty graph.
    let entry_id = uuid::Uuid::new_v4();
    let hexstamp = format!("0x{}", &source_hash[..16]); // truncated fingerprint

    let donut_entry = DonutloopEntry {
        entry_id,
        prev_hexstamp: None, // you would look up the last entry and link it here
        hexstamp,
        proposal_id: Some("anomaly-batch-1".into()),
        decision,
        roh_before: 0.0,
        roh_after: max_roh,
        accounted_anomalies: anomaly_count,
        subject_did: subject_did.clone(),
        timestamp: Utc::now(),
        policy_refs: vec![
            "rohmodel.aln".into(),
            "neurorights.json".into(),
            "stake.aln".into(),
        ],
    };

    // 7. Bundle into a batch and write to ALN‑style files.
    let batch = AnalysisBatch {
        batch_id: uuid::Uuid::new_v4(),
        created_at: Utc::now(),
        subject_did,
        provider: "google".into(),
        events,
        assessments,
        biosession_shard,
        donut_entry,
    };

    batch.write_to_aln_files("sovereign-logs")?;

    Ok(())
}
