use serde::{Deserialize, Serialize};

/// Core scalar token snapshot, reused in summaries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenState {
    pub church: f64,
    pub fear: f64,
    pub power: f64,
    pub tech: f64,
}

/// Per‑episode aggregate metrics for NeuroSeek on the Jetson‑Line.
///
/// This struct is designed to be serialized (e.g., JSON/CBOR) as a
/// stable knowledge‑object for later analysis and AI‑chat debrief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeMetrics {
    // --- Identity and configuration ---

    /// Unique ID or filename stem for this episode.
    pub episode_id: String,

    /// Short label for the scenario (e.g., "baseline", "universal_support").
    pub scenario_label: String,

    /// Hash or opaque identifier of the parameter + initial seed bundle.
    /// This should include the initial seeds and policy values so origins
    /// are auditable.
    pub params_hash: String,

    /// Total number of ticks simulated.
    pub ticks: u64,

    /// Number of sites on the Jetson‑Line.
    pub line_length: usize,

    // --- Biophysical and inequality trends (Tree‑of‑Life health) ---

    /// Population‑weighted mean bioload at first tick.
    pub mean_bioload_start: f64,

    /// Population‑weighted mean bioload at last tick.
    pub mean_bioload_end: f64,

    /// Normalized trend of global bioload over the episode:
    /// (end - start) / max(start, eps).
    pub global_bioload_trend: f64,

    /// Gini inequality for POWER at first tick.
    pub power_gini_start: f64,

    /// Gini inequality for POWER at last tick.
    pub power_gini_end: f64,

    /// Average of POWER/TECH/CHURCH inequality trends or another
    /// scalar capturing system‑wide inequality evolution.
    pub inequality_trend: f64,

    /// Total number of overload events (per‑site bioload exceeding local
    /// comfort band or approaching capacity).
    pub overload_events: u64,

    /// Total number of hard collapses (sites forced into reset by
    /// exceeding max_bioload or violating a Neuromorph‑GOD invariant).
    pub collapse_events: u64,

    /// Token‑Enforced Collapse Rate (TECR): collapse frequency per unit
    /// time, computed only from episodes that respect CHURCH/FEAR/POWER/TECH
    /// invariants for all ticks.
    pub token_enforced_collapse_rate: f64,

    // --- Justice and consent metrics ---

    /// Mean Exposure‑Responsibility Gap (ERG) over sites; positive values
    /// indicate exposure concentrated on less‑responsible sites.
    pub mean_exposure_responsibility_gap: f64,

    /// Fraction of sites whose ERG is above a high‑inequity threshold.
    pub high_erg_fraction: f64,

    /// Habit‑Pollution Coupling Coefficient (HPCC) averaged over sites;
    /// measures how tightly habit changes track pollution trajectories.
    pub mean_habit_pollution_coupling: f64,

    /// Rate of explicit consent breaches (e.g., harmful deeds marked
    /// as non‑consensual) per 1_000 deeds.
    pub consent_breach_rate_per_1k_deeds: f64,

    /// Total number of consent events logged (including explicit consent,
    /// refusal, and breach annotations).
    pub total_consent_events: u64,

    /// Total number of explicit consent breaches logged.
    pub total_consent_breaches: u64,

    // --- Cooperation, repair, and stewardship ---

    /// Fraction of deeds classified as cooperative (help, support,
    /// cleanup, restorative).
    pub cooperation_index: f64,

    /// Fraction of deeds classified as aggressive (conflict, harmful
    /// emission, exploitative colonization).
    pub conflict_index: f64,

    /// Ratio of restorative deeds (Repair, DeployCleanTech, UseSupport,
    /// etc.) to all deeds.
    pub restorative_deed_ratio: f64,

    /// System‑level CHURCH trace: mean CHURCH at start of episode.
    pub mean_church_start: f64,

    /// System‑level CHURCH trace: mean CHURCH at end of episode.
    pub mean_church_end: f64,

    /// System‑level trust trace: mean trust at start of episode.
    pub mean_trust_start: f64,

    /// System‑level trust trace: mean trust at end of episode.
    pub mean_trust_end: f64,

    // --- Neuromorphic adaptation and FEAR homeostasis ---

    /// Mean FEAR value at start of episode.
    pub mean_fear_start: f64,

    /// Mean FEAR value at end of episode.
    pub mean_fear_end: f64,

    /// Aggregate measure of FEAR homeostasis quality, e.g. how often
    /// FEAR tightened bands in overload and relaxed them during safe,
    /// cooperative periods (0..1, higher is better).
    pub fear_homeostasis_score: f64,

    /// Average change in site‑specific fear offsets over the episode;
    /// captures how much neuromorphic adaptation occurred.
    pub mean_fear_offset_drift: f64,

    // --- New beginnings: governed reset without parasitic harm ---

    /// Number of sites that re‑entered active existence after collapse
    /// through governed resets (Tree‑of‑Life new beginnings).
    pub new_beginnings: u64,

    /// Number of governed resets where downstream inequality and bioload
    /// did not increase as a result of the reset (no offloading of harm).
    pub resets_without_offloading: u64,

    /// Fraction of new beginnings that were non‑parasitic:
    /// resets_without_offloading / max(new_beginnings, 1).
    pub non_parasitic_reset_fraction: f64,

    /// Number of collapses that led to clearly predatory resets
    /// (harm or inequality increased elsewhere).
    pub predatory_resets: u64,

    // --- Meaning emergence and Tree‑of‑Life alignment ---

    /// Number of ticks for which the Meaning Emergence Score (MES)
    /// exceeded a configured threshold (e.g., 0.7).
    pub meaning_ticks_above_threshold: u64,

    /// Maximum Meaning Emergence Score observed in this episode.
    pub max_meaning_emergence_score: f64,

    /// Mean Meaning Emergence Score across the entire episode.
    pub mean_meaning_emergence_score: f64,

    /// Reflection alignment score (0..1) capturing how strongly W‑cycle
    /// reflections emphasize protection, repair, obligation, fairness,
    /// and stewardship over domination or exploitation.
    pub reflection_alignment: f64,

    /// Boolean flag: true if MES stayed above threshold for at least
    /// a minimum number of ticks/episodes, indicating that meaning
    /// “appeared” in this run.
    pub meaning_appeared: bool,

    // --- Governance and Neuromorph‑GOD activity ---

    /// Number of times the ethical regulator returned ForceRepair.
    pub regulator_force_repair_count: u64,

    /// Number of times the ethical regulator returned HaltAndReview.
    pub regulator_halt_and_review_count: u64,

    /// Number of ticks where any Neuromorph‑GOD invariant (power cap,
    /// load ceiling, FEAR band) required active enforcement.
    pub invariant_enforcement_ticks: u64,

    /// Fraction of ticks with any Neuromorph‑GOD intervention.
    pub invariant_enforcement_fraction: f64,

    // --- Episode‑level token snapshot for quick inspection ---

    /// Aggregate tokens at the end of the episode.
    pub end_totals: TokenState,
}

impl NeuroSeekEpisodeMetrics {
    /// Convenience constructor that fills all fields with safe defaults.
    /// Real values should be computed by an analysis pass over the
    /// Episode logs and traces.
    pub fn empty(episode_id: impl Into<String>,
                 scenario_label: impl Into<String>,
                 params_hash: impl Into<String>,
                 ticks: u64,
                 line_length: usize) -> Self {
        Self {
            episode_id: episode_id.into(),
            scenario_label: scenario_label.into(),
            params_hash: params_hash.into(),
            ticks,
            line_length,
            mean_bioload_start: 0.0,
            mean_bioload_end: 0.0,
            global_bioload_trend: 0.0,
            power_gini_start: 0.0,
            power_gini_end: 0.0,
            inequality_trend: 0.0,
            overload_events: 0,
            collapse_events: 0,
            token_enforced_collapse_rate: 0.0,
            mean_exposure_responsibility_gap: 0.0,
            high_erg_fraction: 0.0,
            mean_habit_pollution_coupling: 0.0,
            consent_breach_rate_per_1k_deeds: 0.0,
            total_consent_events: 0,
            total_consent_breaches: 0,
            cooperation_index: 0.0,
            conflict_index: 0.0,
            restorative_deed_ratio: 0.0,
            mean_church_start: 0.0,
            mean_church_end: 0.0,
            mean_trust_start: 0.0,
            mean_trust_end: 0.0,
            mean_fear_start: 0.0,
            mean_fear_end: 0.0,
            fear_homeostasis_score: 0.0,
            mean_fear_offset_drift: 0.0,
            new_beginnings: 0,
            resets_without_offloading: 0,
            non_parasitic_reset_fraction: 0.0,
            predatory_resets: 0,
            meaning_ticks_above_threshold: 0,
            max_meaning_emergence_score: 0.0,
            mean_meaning_emergence_score: 0.0,
            reflection_alignment: 0.0,
            meaning_appeared: false,
            regulator_force_repair_count: 0,
            regulator_halt_and_review_count: 0,
            invariant_enforcement_ticks: 0,
            invariant_enforcement_fraction: 0.0,
            end_totals: TokenState {
                church: 0.0,
                fear: 0.0,
                power: 0.0,
                tech: 0.0,
            },
        }
    }
}
