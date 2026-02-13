use crate::neuroseek::episode::*;
use crate::sim::{World, DeedLog, DeedKind, SiteId};
use std::collections::HashMap;

pub fn analyze_episode(world: &World, log: &DeedLog) -> NeuroSeekEpisodeSummary {
    // ------------------------------------------------------------------------
    // 1. Gather per‑site time‑series (if needed for averages over time)
    //    For simplicity, we assume world holds the final state after the episode.
    //    If you want time‑averaged metrics, you would need to store snapshots.
    // ------------------------------------------------------------------------

    // ----- Addiction & exposure ----------------------------------------------
    let occupied_sites: Vec<&Site> = world.sites.iter().filter(|s| s.occupied).collect();
    let n = occupied_sites.len() as f64;

    let habit_loads: Vec<f64> = occupied_sites.iter().map(|s| s.addiction.habit_load).collect();
    let mean_habit_load = habit_loads.iter().sum::<f64>() / n;
    let max_habit_load = habit_loads.iter().cloned().fold(0./0., f64::max); // handle empty
    let habit_load_gini = gini_coefficient(&habit_loads);

    let total_use_habit_deeds = log.deeds.iter().filter(|d| d.kind == DeedKind::UseHabit).count() as u64;
    let total_use_support_deeds = log.deeds.iter().filter(|d| d.kind == DeedKind::UseSupport).count() as u64;

    let exposure_doses: Vec<f64> = occupied_sites.iter().map(|s| s.addiction.exposure_dose).collect();
    let mean_exposure_dose = exposure_doses.iter().sum::<f64>() / n;
    let max_exposure_dose = exposure_doses.iter().cloned().fold(0./0., f64::max);

    // ----- Biophysical harm & recovery ---------------------------------------
    let bioloads: Vec<f64> = occupied_sites.iter().map(|s| s.bioload).collect();
    let mean_bioload = bioloads.iter().sum::<f64>() / n;
    let max_bioload = bioloads.iter().cloned().fold(0./0., f64::max);

    let collapse_events = log.events.iter().filter(|e| e.kind == EventKind::NeuromorphGodIntervention).count() as u64;
    let recovery_events = log.events.iter().filter(|e| e.kind == EventKind::SiteRecovered).count() as u64;

    // ----- Consent & Tree‑of‑FEAR --------------------------------------------
    // These require that NEURO‑CONSENT‑SMOKING was run periodically during the simulation
    // and its outputs stored in the log or in a separate consent_log.
    let consent_respected_events = log.consent_events.iter().filter(|c| c.label == "respectsconsent").count() as u64;
    let consent_breach_events = log.consent_events.iter().filter(|c| c.label == "violatesconsent").count() as u64;
    let addictionpull_events = log.tree_of_fear_events.iter().filter(|t| t.trunk == "ADDICTIONPULL").count() as u64;
    let healingcure_events = log.tree_of_fear_events.iter().filter(|t| t.trunk == "HEALINGCURE").count() as u64;

    let roh_neuro_mean = occupied_sites.iter().map(|s| s.rohweightneuro).sum::<f64>() / n;

    // ----- Fairness / justice ------------------------------------------------
    // hpcc_mean: Human Physical–Civic Cost – could be Gini of (exposure_dose * bioload)
    let harm_product: Vec<f64> = occupied_sites.iter().map(|s| s.exposure_dose * s.bioload).collect();
    let hpcc_mean = gini_coefficient(&harm_product);

    // erg_mean: Energy–Resource–Greed – ratio of POWER/TECH spent on addiction vs repair
    let power_spent_on_addiction = log.deeds.iter()
        .filter(|d| d.kind == DeedKind::UseHabit)
        .map(|d| d.power_cost)
        .sum::<f64>();
    let power_spent_on_repair = log.deeds.iter()
        .filter(|d| matches!(d.kind, DeedKind::UseSupport | DeedKind::DeployCleanTech | DeedKind::RepairEnvironment))
        .map(|d| d.power_cost)
        .sum::<f64>();
    let erg_mean = if power_spent_on_repair > 0.0 {
        power_spent_on_addiction / power_spent_on_repair
    } else {
        f64::INFINITY // pure greed
    };

    // tecr_mean: Tree‑of‑Life Equity and Consent Redemption – composite score
    // Example: (consent_respected_events / (consent_respected_events + consent_breach_events + 1))
    //          * (1.0 - support_access_inequality) * (1.0 - pollution_inequality)
    let consent_ratio = consent_respected_events as f64 / (consent_respected_events + consent_breach_events + 1) as f64;
    let tecr_mean = consent_ratio * (1.0 - support_access_inequality) * (1.0 - pollution_inequality);

    let pollution_stocks: Vec<f64> = occupied_sites.iter().map(|s| s.pollution_stock).collect();
    let pollution_inequality = gini_coefficient(&pollution_stocks);

    // support_access_inequality: Gini of UseSupport deeds per site
    let mut support_per_site: HashMap<SiteId, u64> = HashMap::new();
    for deed in log.deeds.iter().filter(|d| d.kind == DeedKind::UseSupport) {
        *support_per_site.entry(deed.site_id).or_insert(0) += 1;
    }
    let support_counts: Vec<f64> = world.sites.iter().map(|s| *support_per_site.get(&s.id).unwrap_or(&0) as f64).collect();
    let support_access_inequality = gini_coefficient(&support_counts);

    // ----- Forgiveness / redemption ------------------------------------------
    // habit_reduction_after_support: average reduction after UseSupport sequences
    // This requires tracking habit_load changes per site before/after each UseSupport.
    // Simplified: compute average Δhabit for deeds that are UseSupport and where habit decreased.
    let mut total_reduction = 0.0;
    let mut reduction_count = 0;
    for deed in log.deeds.iter().filter(|d| d.kind == DeedKind::UseSupport) {
        if let Some(prev) = log.state_before(deed) {
            let after = log.state_after(deed);
            let delta = prev.habit_load - after.habit_load;
            if delta > 0.0 {
                total_reduction += delta;
                reduction_count += 1;
            }
        }
    }
    let habit_reduction_after_support = if reduction_count > 0 { total_reduction / reduction_count as f64 } else { 0.0 };

    // bioload_reduction_after_repair: similar for RepairEnvironment / SupportCessation
    // (Implementation analogous)

    // redemptive_traj_count: sites that went from high habit+breach to low habit+respected
    let redemptive_traj_count = occupied_sites.iter()
        .filter(|s| {
            let start = log.initial_state(s.id);
            let end = s;
            start.habit_load > HIGH_HABIT_THRESHOLD
                && start.consent_breaches > HIGH_BREACH_THRESHOLD
                && end.habit_load < LOW_HABIT_THRESHOLD
                && end.consent_breaches == 0
        })
        .count() as u64;

    // ------------------------------------------------------------------------
    // 2. Build the metrics struct
    // ------------------------------------------------------------------------
    let metrics = NeuroSeekEpisodeMetrics {
        run_id: world.run_id.clone(),
        seed: world.seed,
        params_hash: world.params_hash.clone(),
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
        bioload_reduction_after_repair,   // compute similarly
        redemptive_traj_count,
    };

    // ------------------------------------------------------------------------
    // 3. Generate reflections (line, clusters, sites, forgiveness)
    //    This can be done with template‑based strings that interpolate metrics.
    //    For a more advanced version, you could use an LLM prompt, but rule‑based
    //    is sufficient for now.
    // ------------------------------------------------------------------------
    let reflections = generate_reflections(&metrics, world, log);

    NeuroSeekEpisodeSummary { metrics, reflections }
}

// Helper: Gini coefficient calculation
fn gini_coefficient(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let sum_abs = sorted.iter().enumerate()
        .map(|(i, &x)| (2*i - n + 1) as f64 * x)
        .sum::<f64>();
    let mean = sorted.iter().sum::<f64>() / n as f64;
    sum_abs / (n as f64 * n as f64 * mean)
}
