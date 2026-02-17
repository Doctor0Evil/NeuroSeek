//! Online clustering of 5D biophysical states into micro-polytopes.
//!
//! Implements an adaptive streaming algorithm with decay, merging, and pruning.
//! Pure Rust, no I/O, no device control—safe for observation-only mode.

use crate::model::{MicroPolytope, TaggedState, BiophysicalState};
use std::time::{Duration, Instant};

/// Configuration for the polytope clusterer.
#[derive(Debug, Clone, Copy)]
pub struct ClustererConfig {
    /// Multiplier for a polytope's radius to determine membership.
    pub radius_factor: f64,
    /// Maximum allowed radius for any polytope (prevents over‑merging).
    pub max_radius: f64,
    /// Per‑second decay rate (e.g., 0.99). Applied to all polytopes over time.
    pub decay_rate: f64,
    /// Distance threshold for merging two polytopes (absolute in normalized space).
    pub merge_threshold: f64,
    /// Minimum weight for a polytope to be kept (pruning threshold).
    pub min_weight: f64,
}

impl Default for ClustererConfig {
    fn default() -> Self {
        Self {
            radius_factor: 1.5,
            max_radius: 0.3,
            decay_rate: 0.99,
            merge_threshold: 0.1,
            min_weight: 2.0,
        }
    }
}

/// Online clusterer that maintains a set of micro-polytopes.
pub struct PolytopeClusterer {
    polytopes: Vec<MicroPolytope>,
    config: ClustererConfig,
    last_maintenance: Instant,
    maintenance_interval: Duration,
}

impl PolytopeClusterer {
    /// Create a new clusterer with given config and maintenance interval.
    pub fn new(config: ClustererConfig, maintenance_interval: Duration) -> Self {
        Self {
            polytopes: Vec::new(),
            config,
            last_maintenance: Instant::now(),
            maintenance_interval,
        }
    }

    /// Insert a new observation point at time `now`.
    pub fn insert_point(&mut self, point: TaggedState) {
        let now = point.timestamp;
        // Find the nearest polytope within its radius * radius_factor.
        let mut best_idx = None;
        let mut best_dist = f64::INFINITY;

        for (i, poly) in self.polytopes.iter().enumerate() {
            let dist = point.state.distance(&poly.centroid());
            let threshold = poly.radius() * self.config.radius_factor;
            if dist <= threshold && dist < best_dist {
                best_idx = Some(i);
                best_dist = dist;
            }
        }

        if let Some(idx) = best_idx {
            // Update existing polytope.
            self.polytopes[idx].update(&point, self.config.decay_rate);
        } else {
            // Create a new polytope.
            self.polytopes.push(MicroPolytope::from_point(&point));
        }

        // Periodic maintenance.
        if now.duration_since(self.last_maintenance) >= self.maintenance_interval {
            self.maintenance(now);
        }
    }

    /// Apply time decay to all polytopes (called internally, but can be exposed if needed).
    pub fn tick_decay(&mut self, now: Instant) {
        for poly in &mut self.polytopes {
            let dt = now.duration_since(poly.last_update).as_secs_f64();
            if dt > 0.0 {
                poly.decay(self.config.decay_rate.powf(dt));
                poly.last_update = now;
            }
        }
    }

    /// Merge polytopes that are very close.
    pub fn merge_close(&mut self) {
        // Sort by weight descending (greedy merging).
        self.polytopes.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

        let mut i = 0;
        while i < self.polytopes.len() {
            let mut j = i + 1;
            while j < self.polytopes.len() {
                let dist = self.polytopes[i].centroid().distance(&self.polytopes[j].centroid());
                if dist < self.config.merge_threshold {
                    // Merge j into i.
                    let (left, right) = self.polytopes.split_at_mut(j);
                    let (poly_i, poly_j) = (&mut left[i], &right[0]);
                    poly_i.weight += poly_j.weight;
                    for k in 0..5 {
                        poly_i.linear_sum[k] += poly_j.linear_sum[k];
                        poly_i.sq_sum[k] += poly_j.sq_sum[k];
                    }
                    poly_i.craving_sum += poly_j.craving_sum;
                    poly_i.craving_count += poly_j.craving_count;
                    // Merge stimulus counts
                    for (id, count) in &poly_j.stimulus_counts {
                        *poly_i.stimulus_counts.entry(id.clone()).or_insert(0) += count;
                    }
                    // Remove poly_j.
                    self.polytopes.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    /// Remove polytopes with weight below `min_weight`.
    pub fn prune(&mut self) {
        self.polytopes.retain(|p| p.weight >= self.config.min_weight);
    }

    /// Perform full maintenance: decay, merge, prune.
    pub fn maintenance(&mut self, now: Instant) {
        self.tick_decay(now);
        self.merge_close();
        self.prune();
        self.last_maintenance = now;
    }

    /// Get a reference to the current polytopes.
    pub fn polytopes(&self) -> &[MicroPolytope] {
        &self.polytopes
    }

    /// Associate a craving event with the polytope containing the given state.
    /// If no polytope contains the state, do nothing.
    pub fn associate_craving(&mut self, state: &BiophysicalState, intensity: f64, now: Instant) {
        if let Some(poly) = self
            .polytopes
            .iter_mut()
            .find(|p| state.distance(&p.centroid()) <= p.radius() * self.config.radius_factor)
        {
            poly.add_craving(intensity);
            poly.last_update = now; // optional
        }
    }

    /// Query stimulus statistics: for a given stimulus_id, return the polytopes it appears in
    /// and the count.
    pub fn stimulus_polytopes(&self, stimulus_id: &str) -> Vec<(&MicroPolytope, u64)> {
        self.polytopes
            .iter()
            .filter_map(|p| p.stimulus_counts.get(stimulus_id).map(|c| (p, *c)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{StimulusMetadata, TaggedState};

    fn test_point(e: f64, m: f64, s: f64, th: f64, t: f64, stim_id: Option<&str>) -> TaggedState {
        let state = BiophysicalState::new(e, m, s, th, t);
        let stimulus = stim_id.map(|id| StimulusMetadata {
            stimulus_id: id.to_string(),
            stimulus_name: "test".into(),
            audio_params: None,
        });
        TaggedState {
            state,
            timestamp: Instant::now(),
            stimulus,
        }
    }

    #[test]
    fn test_insert_with_stimulus() {
        let config = ClustererConfig::default();
        let mut clusterer = PolytopeClusterer::new(config, Duration::from_secs(60));
        let p1 = test_point(0.1, 0.1, 0.1, 0.1, 0.1, Some("stimA"));
        let p2 = test_point(0.11, 0.11, 0.11, 0.11, 0.11, Some("stimA")); // close
        let p3 = test_point(0.9, 0.9, 0.9, 0.9, 0.9, Some("stimB")); // far

        clusterer.insert_point(p1);
        clusterer.insert_point(p2);
        clusterer.insert_point(p3);

        assert_eq!(clusterer.polytopes().len(), 2); // p1+p2 merged, p3 separate
        let poly_a = &clusterer.polytopes()[0]; // should be the merged one
        assert_eq!(poly_a.stimulus_counts.get("stimA"), Some(&2));
        assert_eq!(poly_a.stimulus_counts.get("stimB"), None);

        let poly_b = &clusterer.polytopes()[1];
        assert_eq!(poly_b.stimulus_counts.get("stimB"), Some(&1));
    }
}
