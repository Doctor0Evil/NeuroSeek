use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

pub mod providers;
pub mod roh;
pub mod sovereignty;

/// Provider‑neutral login / access event I ingest into my pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    pub id: Uuid,
    pub provider: String,        // "google", "facebook", "cloudflare", "perplexity"
    pub account_id: String,      // pseudonymous local id I assign, not raw email
    pub timestamp: DateTime<Utc>,
    pub ip: String,
    pub region: Option<String>,
    pub user_agent: Option<String>,
    pub mfa_used: Option<bool>,
    pub success: bool,
    pub raw_source_path: String, // for audit trace
}

/// Axes along which I want anomalies flagged.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnomalyFlags {
    pub impossible_travel: bool,
    pub unusual_region: bool,
    pub unusual_time_of_day: bool,
    pub new_user_agent: bool,
    pub mfa_missing_when_expected: bool,
    pub token_reuse_suspected: bool,
}

/// Sovereign anomaly classification – always host‑centric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAssessment {
    pub event_id: Uuid,
    pub severity: AnomalySeverity,
    pub flags: AnomalyFlags,
    pub roh_score: f32,      // Risk‑of‑Harm in [0,1]
    pub cybostate_ok: bool,  // true only if within my sovereign envelope
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnomalySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for AnomalySeverity {
    fn default() -> Self {
        AnomalySeverity::Info
    }
}

/// ALN‑style shard for a single biosession / account‑session window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiosessionShard {
    pub subject_did: String,   // my DID / Bostrom address binding
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub provider: String,
    pub total_events: u32,
    pub anomaly_count: u32,
    pub max_roh: f32,
    pub cybostate_min: f32,
    pub roh_ceiling: f32,      // usually 0.3 in my doctrine
    pub source_hash: String,   // hash of underlying CSV/logs
}

/// .donutloop‑like ledger entry for sovereignty graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonutloopEntry {
    pub entry_id: Uuid,
    pub prev_hexstamp: Option<String>,
    pub hexstamp: String,

    pub proposal_id: Option<String>, // here: anomaly_batch_id or similar
    pub decision: DonutDecision,     // e.g., Mitigate, Monitor, Ignore
    pub roh_before: f32,
    pub roh_after: f32,
    pub accounted_anomalies: u32,

    pub subject_did: String,
    pub timestamp: DateTime<Utc>,
    pub policy_refs: Vec<String>,    // e.g., ["rohmodel.v1", "neurorights.json"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DonutDecision {
    Mitigate,
    Monitor,
    Ignore,
}

/// A single sovereignty‑guarded analysis batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisBatch {
    pub batch_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub subject_did: String,
    pub provider: String,
    pub events: Vec<AccessEvent>,
    pub assessments: Vec<AnomalyAssessment>,
    pub biosession_shard: BiosessionShard,
    pub donut_entry: DonutloopEntry,
}

impl AnalysisBatch {
    /// Serialize ALN‑style shards and ledger entries to disk in an append‑only fashion.
    pub fn write_to_aln_files<P: AsRef<Path>>(
        &self,
        dir: P,
    ) -> anyhow::Result<()> {
        std::fs::create_dir_all(&dir)?;
        let dir = dir.as_ref();

        // 1. biosession shard as NDJSON
        let biosession_path = dir.join("biosession.aln");
        append_ndjson(&biosession_path, &self.biosession_shard)?;

        // 2. donutloop ledger as NDJSON
        let donut_path = dir.join("donutloop.aln");
        append_ndjson(&donut_path, &self.donut_entry)?;

        // 3. assessments as NDJSON (for future forensics)
        let assess_path = dir.join("anomaly.assessment.aln");
        for a in &self.assessments {
            append_ndjson(&assess_path, a)?;
        }

        Ok(())
    }
}

/// Append a JSON‑serialized line to a file (append‑only ALN/NDJSON style).
fn append_ndjson<T: ?Sized + Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(value)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Compute a SHA‑256 hex digest of raw bytes or concatenated CSV text.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
