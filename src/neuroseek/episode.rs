use serde::{Deserialize, Serialize};

/// Core per-episode metrics for NeuroSeek addiction–consent–fairness runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeMetrics {
    // Identity / provenance
    pub run_id: String,
    pub seed: u64,
    pub params_hash: String,

    // Addiction & exposure band
    pub mean_habit_load: f64,
    pub max_habit_load: f64,
    pub habit_load_gini: f64,
    pub total_use_habit_deeds: u64,
    pub total_use_support_deeds: u64,
    pub mean_exposure_dose: f64,
    pub max_exposure_dose: f64,

    // Biophysical harm & recovery
    pub mean_bioload: f64,
    pub max_bioload: f64,
    pub collapse_events: u64,
    pub recovery_events: u64,

    // Consent & Tree-of-FEAR diagnostics
    pub consent_respected_events: u64,
    pub consent_breach_events: u64,
    pub addictionpull_events: u64,
    pub healingcure_events: u64,
    pub roh_neuro_mean: f64,

    // Fairness / justice metrics
    pub hpcc_mean: f64,
    pub erg_mean: f64,
    pub tecr_mean: f64,
    pub pollution_inequality: f64,
    pub support_access_inequality: f64,

    // Forgiveness / redemption indicators
    pub habit_reduction_after_support: f64,
    pub bioload_reduction_after_repair: f64,
    pub redemptive_traj_count: u64,
}
