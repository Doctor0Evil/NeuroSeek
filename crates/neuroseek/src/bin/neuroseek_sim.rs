//! Offline simulator for NeuroSeek clustering.
//!
//! Generates synthetic 5D biophysical states, feeds them into the clusterer,
//! and periodically outputs JSON snapshots for visualization and tuning.
//! Run with: cargo run --bin neuroseek_sim

use neuroseek::clustering::{ClustererConfig, PolytopeClusterer};
use neuroseek::model::BiophysicalState;
use rand::Rng;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

/// Generate a random state within a specified region.
fn random_state_in_region(rng: &mut impl Rng, center: &BiophysicalState, spread: f64) -> BiophysicalState {
    BiophysicalState::new(
        center.e + rng.gen_range(-spread..spread),
        center.m_prot + rng.gen_range(-spread..spread),
        center.s_bio + rng.gen_range(-spread..spread),
        center.theta + rng.gen_range(-spread..spread),
        center.t + rng.gen_range(-spread..spread),
    )
    .clamp(0.0, 1.0) // keep within normalized range
}

impl BiophysicalState {
    fn clamp(mut self, min: f64, max: f64) -> Self {
        self.e = self.e.clamp(min, max);
        self.m_prot = self.m_prot.clamp(min, max);
        self.s_bio = self.s_bio.clamp(min, max);
        self.theta = self.theta.clamp(min, max);
        self.t = self.t.clamp(min, max);
        self
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let config = ClustererConfig {
        radius_factor: 1.8,
        max_radius: 0.3,
        decay_rate: 0.995,      // slow decay
        merge_threshold: 0.15,
        min_weight: 3.0,
    };
    let maintenance_interval = Duration::from_secs(10); // simulated seconds
    let mut clusterer = PolytopeClusterer::new(config, maintenance_interval);

    // Simulation parameters
    let total_steps = 1000;
    let step_duration = Duration::from_secs(1); // 1 simulated second per step
    let mut rng = rand::thread_rng();
    let start_time = Instant::now();

    // Define a few attractor regions (simulating different biophysical modes)
    let regions = vec![
        (BiophysicalState::new(0.2, 0.3, 0.2, 0.4, 0.3), 0.05), // "calm" region
        (BiophysicalState::new(0.7, 0.6, 0.8, 0.3, 0.2), 0.08), // "stressed" region
        (BiophysicalState::new(0.5, 0.5, 0.5, 0.5, 0.5), 0.1),  // "neutral" region
    ];

    // Create output directory
    let out_dir = Path::new("neuroseek_sim_output");
    std::fs::create_dir_all(out_dir)?;

    for step in 0..total_steps {
        let now = start_time + step * step_duration;

        // Generate a point by picking a random region and adding noise
        let (center, spread) = &regions[rng.gen_range(0..regions.len())];
        let point = random_state_in_region(&mut rng, center, *spread);

        // Insert into clusterer
        clusterer.insert_point(point, now);

        // Occasionally output a snapshot
        if step % 100 == 0 || step == total_steps - 1 {
            let snapshot = json!({
                "step": step,
                "time_sec": step * step_duration.as_secs(),
                "polytopes": clusterer.polytopes().iter().map(|p| {
                    json!({
                        "id": p.id.to_string(),
                        "weight": p.weight,
                        "centroid": [
                            p.centroid().e,
                            p.centroid().m_prot,
                            p.centroid().s_bio,
                            p.centroid().theta,
                            p.centroid().t
                        ],
                        "radius": p.radius(),
                        "avg_craving": p.avg_craving(),
                    })
                }).collect::<Vec<_>>(),
            });

            let filename = out_dir.join(format!("snapshot_{:04}.json", step));
            let mut file = File::create(&filename)?;
            writeln!(file, "{}", serde_json::to_string_pretty(&snapshot)?)?;
            println!("Wrote snapshot to {}", filename.display());
        }
    }

    println!("Simulation complete. {} polytopes remain.", clusterer.polytopes().len());
    Ok(())
}
