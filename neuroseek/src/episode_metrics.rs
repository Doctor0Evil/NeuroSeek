use serde::{Deserialize, Serialize};

/// Snapshot of aggregate token totals at a given moment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenTotals {
    pub church: f64,
    pub fear: f64,
    pub power: f64,
    pub tech: f64,
}

/// Per‑episode aggregate metrics for NeuroSeek on the Jetson‑Line.
///
/// This struct is the single, auditable knowledge object produced after each
/// simulation run. All fields are derived from the deed log, periodic state
/// snapshots, and structured W‑cycle reflections. It is designed to be
/// serialised (JSON/CBOR) and stored under `neuroseek/episodes/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroSeekEpisodeMetrics {
    // --- Identity and configuration -----------------------------------------
    /// Unique identifier for this episode (matches filename stem).
    pub episode_id: String,
    /// Human‑readable label for the scenario (e.g., "baseline", "universal_support").
    pub scenario_label: String,
    /// Hash of the full parameter set (initial seeds, policy knobs, invariants).
    /// Enables reproducibility and audit of the exact configuration.
    pub params_hash: String,
    /// Total number of ticks simulated.
    pub ticks: u64,
    /// Number of sites on the Jetson‑Line.
    pub line_length: usize,

    // --- Biophysical health and inequality trends (Tree‑of‑Life health) ----
    /// Population‑weighted mean bioload at first tick.
    pub mean_bioload_start: f64,
    /// Population‑weighted mean bioload at last tick.
    pub mean_bioload_end: f64,
    /// Normalised trend of global bioload: (end - start) / max(start, 1e-9).
    /// Negative values indicate improvement.
    pub global_bioload_trend: f64,

    /// Gini coefficient of POWER at first tick.
    pub power_gini_start: f64,
    /// Gini coefficient of POWER at last tick.
    pub power_gini_end: f64,

    /// Gini coefficient of pollution stock (E_i) at episode end.
    /// Measures inequality in who bears the pollution burden.
    pub pollution_gini: f64,

    /// Gini coefficient of UseSupport deeds per site (over the whole episode).
    /// High values mean healing technology is concentrated among a few sites.
    pub support_access_gini: f64,

    /// Average of normalised inequality trends for POWER, pollution, and support.
    /// A positive value indicates growing inequality overall.
    pub inequality_trend: f64,

    // --- Addiction‑specific raw metrics -------------------------------------
    /// Mean habit load (H_i) across occupied sites.
    pub mean_habit_load: f64,
    /// Maximum habit load observed at any site.
    pub max_habit_load: f64,
    /// Gini coefficient of habit load across occupied sites.
    pub habit_load_gini: f64,
    /// Total number of UseHabit deeds logged.
    pub total_use_habit_deeds: u64,
    /// Total number of UseSupport deeds logged.
    pub total_use_support_deeds: u64,
    /// Mean cumulative exposure dose (D_i) across occupied sites.
    pub mean_exposure_dose: f64,
    /// Maximum cumulative exposure dose at any site.
    pub max_exposure_dose: f64,
    /// Number of exposure events tagged with the ADDICTIONPULL trunk.
    pub addictionpull_events: u64,
    /// Number of exposure events tagged with the HEALINGCURE trunk.
    pub healingcure_events: u64,
    /// Mean rohweightneuro (Tree‑of‑FEAR weight) across subjects/sites.
    pub roh_neuro_mean: f64,
    /// Number of sites that transitioned from a state of high habit load AND
    /// consent breaches to low habit load AND consent respected by episode end.
    pub redemptive_traj_count: u64,

    // --- Stress, collapse, and recovery -------------------------------------
    /// Total number of overload events (per‑site bioload exceeding local comfort band).
    pub overload_events: u64,
    /// Total number of hard collapses (sites forced into reset by exceeding
    /// max_bioload or violating a Neuromorph‑GOD invariant).
    pub collapse_events: u64,
    /// Rate of hard collapses per 1000 ticks (collapse_events * 1000 / ticks).
    pub collapse_rate_per_1k_ticks: f64,

    // --- Justice, consent, and fairness -------------------------------------
    /// Mean Exposure‑Responsibility Gap (ERG) over sites. Positive values
    /// indicate exposure concentrated on less‑responsible sites.
    pub mean_exposure_responsibility_gap: f64,
    /// Fraction of sites whose ERG exceeds a high‑inequity threshold.
    pub high_erg_fraction: f64,
    /// Mean Habit‑Pollution Coupling Coefficient (HPCC) over sites.
    /// Measures how tightly habit changes track pollution trajectories.
    pub mean_habit_pollution_coupling: f64,

    /// Total number of consent events logged (including explicit consent,
    /// refusal, and breach annotations).
    pub total_consent_events: u64,
    /// Total number of explicit consent breaches logged.
    pub total_consent_breaches: u64,
    /// Rate of consent breaches per 1000 deeds.
    pub consent_breach_rate_per_1k_deeds: f64,

    /// Tree‑of‑Life Equity and Consent Redemption (TECR) score.
    /// Composite measure (0..1) combining consent respect, bioload distribution,
    /// and support access. Higher is better.
    /// Computed as:
    ///   (consent_respected_events / (total_consent_events + 1))
    ///   * (1.0 - support_access_gini)
    ///   * (1.0 - pollution_gini)
    pub tecr_score: f64,

    // --- Cooperation, repair, and stewardship -------------------------------
    /// Fraction of deeds classified as cooperative (help, support, cleanup).
    pub cooperation_index: f64,
    /// Fraction of deeds classified as aggressive (conflict, harmful emission,
    /// exploitative colonisation).
    pub conflict_index: f64,
    /// Ratio of restorative deeds (Repair, DeployCleanTech, UseSupport, etc.)
    /// to all deeds.
    pub restorative_deed_ratio: f64,

    /// Mean CHURCH across sites at episode start.
    pub mean_church_start: f64,
    /// Mean CHURCH across sites at episode end.
    pub mean_church_end: f64,
    /// Mean trust (derived from CHURCH/FEAR dynamics) at episode start.
    pub mean_trust_start: f64,
    /// Mean trust at episode end.
    pub mean_trust_end: f64,

    // --- Neuromorphic adaptation and FEAR homeostasis -----------------------
    /// Mean FEAR across sites at episode start.
    pub mean_fear_start: f64,
    /// Mean FEAR across sites at episode end.
    pub mean_fear_end: f64,
    /// Quality of FEAR homeostasis (0..1). Measures how often FEAR tightened
    /// bands during overload and relaxed them during safe, cooperative periods.
    pub fear_homeostasis_score: f64,
    /// Average change in site‑specific fear offsets over the episode.
    /// Captures the amount of neuromorphic adaptation.
    pub mean_fear_offset_drift: f64,

    // --- New beginnings: governed reset without parasitic harm --------------
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

    // --- Meaning emergence and Tree‑of‑Life alignment -----------------------
    /// Number of ticks for which the Meaning Emergence Score (MES)
    /// exceeded a configured threshold (e.g., 0.7).
    pub meaning_ticks_above_threshold: u64,
    /// Maximum Meaning Emergence Score observed in this episode.
    pub max_meaning_emergence_score: f64,
    /// Mean Meaning Emergence Score across the entire episode.
    pub mean_meaning_emergence_score: f64,
    /// Reflection alignment score (0..1) capturing how strongly W‑cycle
    /// reflections emphasise protection, repair, obligation, fairness,
    /// and stewardship over domination or exploitation.
    pub reflection_alignment: f64,
    /// Boolean flag: true if MES stayed above threshold for at least
    /// a minimum number of ticks/episodes, indicating that meaning
    /// “appeared” in this run.
    pub meaning_appeared: bool,

    // --- Governance and Neuromorph‑GOD activity -----------------------------
    /// Number of times the ethical regulator returned ForceRepair.
    pub regulator_force_repair_count: u64,
    /// Number of times the ethical regulator returned HaltAndReview.
    pub regulator_halt_and_review_count: u64,
    /// Number of ticks where any Neuromorph‑GOD invariant (power cap,
    /// load ceiling, FEAR band) required active enforcement.
    pub invariant_enforcement_ticks: u64,
    /// Fraction of ticks with any Neuromorph‑GOD intervention.
    pub invariant_enforcement_fraction: f64,

    // --- Macro token snapshots ----------------------------------------------
    /// Aggregate tokens at episode start.
    pub start_totals: TokenTotals,
    /// Aggregate tokens at episode end.
    pub end_totals: TokenTotals,
}

impl NeuroSeekEpisodeMetrics {
    /// Convenience constructor that fills all fields with safe defaults.
    /// Real values should be computed by an analysis pass over the
    /// Episode logs and traces.
    pub fn empty(
        episode_id: impl Into<String>,
        scenario_label: impl Into<String>,
        params_hash: impl Into<String>,
        ticks: u64,
        line_length: usize,
    ) -> Self {
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
            pollution_gini: 0.0,
            support_access_gini: 0.0,
            inequality_trend: 0.0,

            mean_habit_load: 0.0,
            max_habit_load: 0.0,
            habit_load_gini: 0.0,
            total_use_habit_deeds: 0,
            total_use_support_deeds: 0,
            mean_exposure_dose: 0.0,
            max_exposure_dose: 0.0,
            addictionpull_events: 0,
            healingcure_events: 0,
            roh_neuro_mean: 0.0,
            redemptive_traj_count: 0,

            overload_events: 0,
            collapse_events: 0,
            collapse_rate_per_1k_ticks: 0.0,

            mean_exposure_responsibility_gap: 0.0,
            high_erg_fraction: 0.0,
            mean_habit_pollution_coupling: 0.0,
            total_consent_events: 0,
            total_consent_breaches: 0,
            consent_breach_rate_per_1k_deeds: 0.0,
            tecr_score: 0.0,

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

            start_totals: TokenTotals { church: 0.0, fear: 0.0, power: 0.0, tech: 0.0 },
            end_totals: TokenTotals { church: 0.0, fear: 0.0, power: 0.0, tech: 0.0 },
        }
    }
}
