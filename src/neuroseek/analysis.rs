use crate::neuroseek::episode::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

// ---- Traits to adapt existing World / logs into NeuroSeek analysis ---------

/// Minimal site view needed for metrics.
pub trait NeuroSeekSiteView {
    fn id(&self) -> u64;
    fn occupied(&self) -> bool;
    fn habit_load(&self) -> f64;
    fn exposure_dose(&self) -> f64;
    fn bioload(&self) -> f64;
    fn pollution_stock(&self) -> f64;
    fn rohweightneuro(&self) -> f64;
    fn consent_breaches(&self) -> u64;
    fn use_support_count(&self) -> u64;
}

/// Minimal world view needed.
pub trait NeuroSeekWorldView {
    type Site: NeuroSeekSiteView;

    fn run_id(&self) -> &str;
    fn seed(&self) -> u64;
    fn params_hash(&self) -> &str;
    fn sites(&self) -> &[Self::Site];
}

/// Deed kinds relevant for addiction / repair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuroSeekDeedKind {
    UseHabit,
    UseSupport,
    DeployCleanTech,
    RepairEnvironment,
    Other,
}

/// Minimal per-deed view.
pub trait NeuroSeekDeedView {
    fn kind(&self) -> NeuroSeekDeedKind;
    fn site_id(&self) -> u64;
    fn power_cost(&self) -> f64;
}

/// Minimal consent / Tree-of-FEAR event views.
pub trait NeuroSeekConsentEventView {
    fn label(&self) -> &str; // "respectsconsent" or "violatesconsent"
}

pub trait NeuroSeekTreeOfFearEventView {
    fn trunk(&self) -> &str; // "ADDICTIONPULL", "HEALINGCURE", etc.
}

/// Log aggregation view.
pub trait NeuroSeekLogView {
    type Deed: NeuroSeekDeedView;
    type ConsentEvent: NeuroSeekConsentEventView;
    type TreeOfFearEvent: NeuroSeekTreeOfFearEventView;

    fn deeds(&self) -> &[Self::Deed];
    fn consent_events(&self) -> &[Self::ConsentEvent];
    fn tree_of_fear_events(&self) -> &[Self::TreeOfFearEvent];

    /// Optional: mean bioload reduction after repair; return 0.0 if not tracked.
    fn bioload_reduction_after_repair(&self) -> f64;

    /// Optional: redemptive trajectory count; return 0 if not tracked.
    fn redemptive_traj_count(&self) -> u64;
}

// ---- Thresholds for non-savage evolution -----------------------------------

const MAX_ERG: f64 = 2.0;
const MAX_POLLUTION_GINI: f64 = 0.6;
const MAX_SUPPORT_GINI: f64 = 0.4;
const MIN_TECR: f64 = 0.3;
const MAX_CONSENT_BREACH_RATIO: f64 = 0.10;

// ---- Public analysis entrypoint --------------------------------------------

pub fn analyze_episode<W, L>(world: &W, log: &L) -> NeuroSeekEpisodeSummary
where
    W: NeuroSeekWorldView,
    L: NeuroSeekLogView,
{
    // Collect occupied sites.
    let occupied: Vec<&W::Site> = world
        .sites()
        .iter()
        .filter(|s| s.occupied())
        .collect();
    let n = occupied.len() as f64;
    let n_safe = if n > 0.0 { n } else { 1.0 };

    // Addiction & exposure.
    let habit_loads: Vec<f64> = occupied.iter().map(|s| s.habit_load()).collect();
    let mean_habit_load = habit_loads.iter().sum::<f64>() / n_safe;
    let max_habit_load = habit_loads.iter().cloned().fold(0.0, f64::max);
    let habit_load_gini = gini_coefficient(&habit_loads);

    let total_use_habit_deeds = log
        .deeds()
        .iter()
        .filter(|d| d.kind() == NeuroSeekDeedKind::UseHabit)
        .count() as u64;
    let total_use_support_deeds = log
        .deeds()
        .iter()
        .filter(|d| d.kind() == NeuroSeekDeedKind::UseSupport)
        .count() as u64;

    let exposure_doses: Vec<f64> = occupied.iter().map(|s| s.exposure_dose()).collect();
    let mean_exposure_dose = exposure_doses.iter().sum::<f64>() / n_safe;
    let max_exposure_dose = exposure_doses.iter().cloned().fold(0.0, f64::max);

    // Biophysical harm & recovery.
    let bioloads: Vec<f64> = occupied.iter().map(|s| s.bioload()).collect();
    let mean_bioload = bioloads.iter().sum::<f64>() / n_safe;
    let max_bioload = bioloads.iter().cloned().fold(0.0, f64::max);

    // Here we treat collapse/recovery counts as zero; you can wire them later.
    let collapse_events: u64 = 0;
    let recovery_events: u64 = 0;

    // Consent & Tree-of-FEAR.
    let consent_respected_events = log
        .consent_events()
        .iter()
        .filter(|c| c.label() == "respectsconsent")
        .count() as u64;
    let consent_breach_events = log
        .consent_events()
        .iter()
        .filter(|c| c.label() == "violatesconsent")
        .count() as u64;

    let addictionpull_events = log
        .tree_of_fear_events()
        .iter()
        .filter(|t| t.trunk() == "ADDICTIONPULL")
        .count() as u64;
    let healingcure_events = log
        .tree_of_fear_events()
        .iter()
        .filter(|t| t.trunk() == "HEALINGCURE")
        .count() as u64;

    let roh_neuro_mean = occupied
        .iter()
        .map(|s| s.rohweightneuro())
        .sum::<f64>()
        / n_safe;

    // Fairness / justice.
    let harm_product: Vec<f64> = occupied
        .iter()
        .map(|s| s.exposure_dose() * s.bioload())
        .collect();
    let hpcc_mean = gini_coefficient(&harm_product);

    let power_spent_on_addiction: f64 = log
        .deeds()
        .iter()
        .filter(|d| d.kind() == NeuroSeekDeedKind::UseHabit)
        .map(|d| d.power_cost())
        .sum();
    let power_spent_on_repair: f64 = log
        .deeds()
        .iter()
        .filter(|d| matches!(d.kind(), NeuroSeekDeedKind::UseSupport | NeuroSeekDeedKind::DeployCleanTech | NeuroSeekDeedKind::RepairEnvironment))
        .map(|d| d.power_cost())
        .sum();

    let erg_mean = if power_spent_on_repair > 0.0 {
        power_spent_on_addiction / power_spent_on_repair
    } else if power_spent_on_addiction > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let pollution_stocks: Vec<f64> = occupied.iter().map(|s| s.pollution_stock()).collect();
    let pollution_inequality = gini_coefficient(&pollution_stocks);

    let mut support_per_site: HashMap<u64, u64> = HashMap::new();
    for deed in log.deeds().iter().filter(|d| d.kind() == NeuroSeekDeedKind::UseSupport) {
        *support_per_site.entry(deed.site_id()).or_insert(0) += 1;
    }
    let support_counts: Vec<f64> = occupied
        .iter()
        .map(|s| *support_per_site.get(&s.id()).unwrap_or(&0) as f64)
        .collect();
    let support_access_inequality = gini_coefficient(&support_counts);

    let consent_total = consent_respected_events + consent_breach_events;
    let consent_ratio = if consent_total > 0 {
        consent_respected_events as f64 / consent_total as f64
    } else {
        1.0
    };
    let tecr_mean = consent_ratio * (1.0 - support_access_inequality) * (1.0 - pollution_inequality);

    // Forgiveness / redemption.
    let habit_reduction_after_support = compute_habit_reduction_after_support::<W, L>(world, log);
    let bioload_reduction_after_repair = log.bioload_reduction_after_repair();
    let redemptive_traj_count = log.redemptive_traj_count();

    let metrics = NeuroSeekEpisodeMetrics {
        run_id: world.run_id().to_string(),
        seed: world.seed(),
        params_hash: world.params_hash().to_string(),
        mean_habit_load,
        max_habit_load,
        habit_load_gini,
        total_use_habit_deeds,
        total_use_support_deeds,
        mean_exposure_dose,
        max_exposure_dose,
        mean_bioload,
        max_bioload,
        collapse_events,
        recovery_events,
        consent_respected_events,
        consent_breach_events,
        addictionpull_events,
        healingcure_events,
        roh_neuro_mean,
        hpcc_mean,
        erg_mean,
        tecr_mean,
        pollution_inequality,
        support_access_inequality,
        habit_reduction_after_support,
        bioload_reduction_after_repair,
        redemptive_traj_count,
    };

    let reflections = generate_reflections(&metrics, world, log);

    NeuroSeekEpisodeSummary { metrics, reflections }
}

// ---- Helper: compute mean habit reduction after UseSupport ------------------

fn compute_habit_reduction_after_support<W, L>(_world: &W, _log: &L) -> f64
where
    W: NeuroSeekWorldView,
    L: NeuroSeekLogView,
{
    // Placeholder: return 0.0 until you wire in state-before/state-after access.
    0.0
}

// ---- Helper: Gini coefficient ----------------------------------------------

fn gini_coefficient(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let sum: f64 = sorted.iter().sum();
    if sum == 0.0 {
        return 0.0;
    }
    let mut cumulative = 0.0;
    let mut weighted_sum = 0.0;
    for (i, v) in sorted.iter().enumerate() {
        cumulative += *v;
        weighted_sum += cumulative;
        if i == n - 1 {
            // nothing
        }
    }
    let fair_area = sum * n as f64 / 2.0;
    let lorenz_area = weighted_sum;
    (fair_area - lorenz_area) / fair_area
}

// ---- Reflections generation ------------------------------------------------

fn generate_reflections<W, L>(
    metrics: &NeuroSeekEpisodeMetrics,
    world: &W,
    _log: &L,
) -> NeuroSeekReflections
where
    W: NeuroSeekWorldView,
    L: NeuroSeekLogView,
{
    let line_what = format!(
        "Mean habit load {:.3}, peak {:.3}, pollution Gini {:.3}, consent breaches {}.",
        metrics.mean_habit_load,
        metrics.max_habit_load,
        metrics.pollution_inequality,
        metrics.consent_breach_events
    );

    let line_so_what = if metrics.erg_mean > MAX_ERG {
        "Excess power spent on addiction versus repair indicates a predatory pattern against the Tree-of-Life."
            .to_string()
    } else if metrics.support_access_inequality > MAX_SUPPORT_GINI {
        "Support access is concentrated; vulnerable sites carry more harm than help, which is unfair."
            .to_string()
    } else if metrics.pollution_inequality > MAX_POLLUTION_GINI {
        "Pollution load is uneven; some sites are sacrificed for others' comfort."
            .to_string()
    } else {
        "The line shows partial stewardship; addiction and harm are present but not allowed to dominate."
            .to_string()
    };

    let line_now_what = "Next episodes should tighten emission caps, reward cleanup and support deeds, and keep consent breaches below the allowed band."
        .to_string();

    // Simple cluster/site reflections: here we just take first few sites as examples.
    let sites_view = world.sites();
    let mut site_reflections = Vec::new();
    for s in sites_view.iter().filter(|s| s.occupied()).take(3) {
        let what = format!(
            "Site {} had habit {:.3}, exposure {:.3}, bioload {:.3}.",
            s.id(),
            s.habit_load(),
            s.exposure_dose(),
            s.bioload()
        );
        let so_what = "This site experiences material impact from habit and exposure; without support it risks unfair collapse."
            .to_string();
        let now_what = "Future runs should ensure this site has access to UseSupport and RepairEnvironment before thresholds are crossed."
            .to_string();
        site_reflections.push(WCycleReflection {
            what,
            so_what,
            now_what,
        });
    }

    let clusters: Vec<WCycleReflection> = Vec::new();

    let forgiveness = ForgivenessReflection {
        what_attempts: format!(
            "{} support deeds were attempted; mean habit reduction metric {:.3}.",
            metrics.total_use_support_deeds, metrics.habit_reduction_after_support
        ),
        so_what_evidence: if metrics.redemptive_traj_count > 0 {
            format!(
                "{} sites moved from high-habit, high-harm patterns into low-habit, consent-respecting patterns.",
                metrics.redemptive_traj_count
            )
        } else {
            "No site completed a full redemptive trajectory in this episode.".to_string()
        },
        now_what_policy:
            "Policies should treat genuine reduction of externalised harm as grounds for forgiveness, once evidence of lasting repair is present."
                .to_string(),
    };

    NeuroSeekReflections {
        line: WCycleReflection {
            what: line_what,
            so_what: line_so_what,
            now_what: line_now_what,
        },
        clusters,
        sites: site_reflections,
        forgiveness,
    }
}

// ---- Index management ------------------------------------------------------

pub fn update_index(
    run_id: &str,
    seed: u64,
    params_hash: &str,
    description: Option<String>,
) -> std::io::Result<()> {
    let index_path = Path::new("neuroseek/episodes/index.json");
    let mut index: NeuroSeekEpisodeIndex = if index_path.exists() {
        let file = File::open(index_path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_default()
    } else {
        Vec::new()
    };

    let timestamp_utc = chrono::Utc::now().to_rfc3339();

    let entry = NeuroSeekEpisodeIndexEntry {
        run_id: run_id.to_string(),
        timestamp_utc,
        seed,
        params_hash: params_hash.to_string(),
        description,
        file_path: format!("neuroseek/episodes/episode_{}.json", run_id),
    };
    index.push(entry);

    let temp_path = index_path.with_extension("tmp");
    {
        let file = File::create(&temp_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &index)?;
    }
    fs::rename(temp_path, index_path)?;
    Ok(())
}
