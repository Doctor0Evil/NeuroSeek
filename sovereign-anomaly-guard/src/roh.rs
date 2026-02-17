use crate::{AccessEvent, AnomalyFlags};
use crate::sovereignty::BaselineProfile;

/// Very simple placeholder RoH estimator – you can refine this with your corridor math.
pub fn estimate_roh(
    ev: &AccessEvent,
    flags: &AnomalyFlags,
    baseline: &BaselineProfile,
) -> f32 {
    let mut roh = 0.0;

    if !ev.success {
        return 0.05; // low by default for failed attempts
    }

    if flags.impossible_travel {
        roh += 0.4;
    }
    if flags.unusual_region {
        roh += 0.2;
    }
    if flags.unusual_time_of_day {
        roh += 0.1;
    }
    if flags.new_user_agent {
        roh += 0.1;
    }
    if flags.mfa_missing_when_expected {
        roh += 0.2;
    }
    if flags.token_reuse_suspected {
        roh += 0.3;
    }

    // Clamp to [0,1]
    roh = roh.min(1.0);

    // Never let the analysis tool claim lower risk than my configured floor.
    roh.max(baseline.min_roh_floor)
}
