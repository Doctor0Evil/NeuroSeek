//! Episode schema for NeuroSeek addiction/consent/fairness experiments.
//! Every simulation run produces one `NeuroSeekEpisodeSummary` which is
//! serialised to `neuroseek/episodes/episode_<run_id>.json`.

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
//  METRICS – quantitative, derived entirely from simulation logs & state
// ----------------------------------------------------------------------------

/// Core per‑episode metrics for NeuroSeek.
/// All fields are computed **after** the simulation ends, from the global deed
/// ledger and the final (or averaged) state of the Jetson‑Line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeMetrics {
    // ----- IDENTITY / PROVENANCE ---------------------------------------------
    /// Unique identifier for this run (must match filename).
    pub run_id: String,
    /// PRNG seed used for the simulation.
    pub seed: u64,
    /// Hash of all policy parameters, Neuromorph‑GOD invariants, and initial
    /// conditions. Enables exact reproducibility.
    pub params_hash: String,

    // ----- ADDICTION & EXPOSURE BAND -----------------------------------------
    /// Mean habit load (H_i) across occupied sites.
    pub mean_habit_load: f64,
    /// Maximum habit load observed at any site.
    pub max_habit_load: f64,
    /// Gini coefficient of habit load inequality (0 = equal, 1 = maximally unequal).
    pub habit_load_gini: f64,
    /// Total number of `UseHabit` deeds executed during the episode.
    pub total_use_habit_deeds: u64,
    /// Total number of `UseSupport` deeds executed.
    pub total_use_support_deeds: u64,
    /// Mean cumulative exposure dose (D_i) across sites.
    pub mean_exposure_dose: f64,
    /// Maximum cumulative exposure dose at any site.
    pub max_exposure_dose: f64,

    // ----- BIOPHYSICAL HARM & RECOVERY ---------------------------------------
    /// Mean bioload (stress measure) across sites.
    pub mean_bioload: f64,
    /// Maximum bioload observed.
    pub max_bioload: f64,
    /// Number of times Neuromorph‑GOD had to intervene (forced repair / freeze).
    pub collapse_events: u64,
    /// Number of times an overloaded site returned to a safe band via
    /// `RepairEnvironment`, `SupportCessation`, or `UseSupport`.
    pub recovery_events: u64,

    // ----- CONSENT & TREE‑OF‑FEAR DIAGNOSTICS --------------------------------
    /// Count of exposure windows where NEURO‑CONSENT‑SMOKING emitted `respectsconsent`.
    pub consent_respected_events: u64,
    /// Count of exposure windows where NEURO‑CONSENT‑SMOKING emitted `violatesconsent`.
    pub consent_breach_events: u64,
    /// Number of exposure events tagged with the `ADDICTIONPULL` trunk.
    pub addictionpull_events: u64,
    /// Number of exposure events tagged with the `HEALINGCURE` trunk.
    pub healingcure_events: u64,
    /// Mean `rohweightneuro` (Tree‑of‑FEAR weight) across subjects / sites.
    pub roh_neuro_mean: f64,

    // ----- FAIRNESS / JUSTICE METRICS ----------------------------------------
    /// Human Physical–Civic Cost index – measures how much harm is concentrated.
    /// (e.g., Gini of pollution damage).
    pub hpcc_mean: f64,
    /// Energy–Resource–Greed index – proportion of POWER/TECH spent on
    /// addiction vs repair.
    pub erg_mean: f64,
    /// Tree‑of‑Life Equity and Consent Redemption score – composite of
    /// consent respect, bioload distribution, and support access.
    pub tecr_mean: f64,
    /// Gini coefficient of pollution stock (E_i) – who breathes the smoke?
    pub pollution_inequality: f64,
    /// Gini coefficient of `UseSupport` deeds – did only high‑POWER nodes get help?
    pub support_access_inequality: f64,

    // ----- FORGIVENESS / REDEMPTION INDICATORS -------------------------------
    /// Average reduction in habit load following a `UseSupport` sequence
    /// (only positive reductions).
    pub habit_reduction_after_support: f64,
    /// Average reduction in bioload following `RepairEnvironment` /
    /// `SupportCessation` deeds.
    pub bioload_reduction_after_repair: f64,
    /// Number of sites that transitioned from high habit + breaches
    /// to low habit + consent respected by episode end.
    pub redemptive_traj_count: u64,
}

// ----------------------------------------------------------------------------
//  REFLECTIONS – structured W³ cycles and forgiveness lens
// ----------------------------------------------------------------------------

/// A complete What / So what / Now what reflection.
/// These are generated from metrics/logs and stored as structured text so they
/// can be versioned and traced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WCycleReflection {
    /// What (biophysical fact, directly from metrics/logs).
    pub what: String,
    /// So what (moral / Tree‑of‑Life interpretation).
    pub so_what: String,
    /// Now what (actionable change for future episodes).
    pub now_what: String,
}

/// Forgiveness lens – examines whether attempted repair actually reduced harm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgivenessReflection {
    /// Which sites attempted change, and how much damage was reduced.
    pub what_attempts: String,
    /// Evidence that redemptive deeds reduced global bioload and breaches.
    pub so_what_evidence: String,
    /// Conditions under which similar deeds will be treated as forgiven.
    pub now_what_policy: String,
}

/// All reflections attached to a NeuroSeek episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekReflections {
    /// Whole‑line W‑cycle (global perspective).
    pub line: WCycleReflection,
    /// Cluster‑level reflections (e.g., for “smoker corridors”).
    pub clusters: Vec<WCycleReflection>,
    /// Site‑level reflections for key high‑habit or high‑breach sites.
    pub sites: Vec<WCycleReflection>,
    /// Explicit forgiveness analysis for this episode.
    pub forgiveness: ForgivenessReflection,
}

// ----------------------------------------------------------------------------
//  EPISODE SUMMARY – the top‑level object stored in each JSON file
// ----------------------------------------------------------------------------

/// Complete record of one NeuroSeek simulation run.
/// This is the only struct that should be serialised to disk for an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeSummary {
    pub metrics: NeuroSeekEpisodeMetrics,
    pub reflections: NeuroSeekReflections,
}

/// Index entry for on‑disk episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeIndexEntry {
    pub run_id: String,
    pub timestamp_utc: String,
    pub seed: u64,
    pub params_hash: String,
    pub description: Option<String>,
    pub file_path: String,
}

/// Index is a simple list of entries.
pub type NeuroSeekEpisodeIndex = Vec<NeuroSeekEpisodeIndexEntry>;
