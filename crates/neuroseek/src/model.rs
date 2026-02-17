//! Core data types for NeuroSeek: 5D biophysical state and micro-polytopes.
//!
//! This module defines the outer-domain signals only—no raw neural data.
//! All coordinates are mappable to the nicotine safety schema (HostBudget,
//! BioCompatibilityEnvelope) and are used purely for observation and discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A point in the 5D bioscale space used for NeuroSeek clustering.
///
/// Dimensions (intentionally aligned with nicotine safety state):
/// - `E`: Energy load / metabolic demand (normalized 0..1)
/// - `M_prot`: Protein burden / oxidative stress marker (normalized)
/// - `S_bio`: Biostress index (composite from HRV, EDA, etc.)
/// - `theta`: Brain state proxy (e.g., theta/delta ratio, normalized)
/// - `T`: Skin temperature deviation from baseline (°C, scaled)
///
/// All values should be normalized to comparable ranges (e.g., 0..1) for
/// distance calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BiophysicalState {
    pub e: f64,        // E
    pub m_prot: f64,   // M_prot
    pub s_bio: f64,    // S_bio
    pub theta: f64,    // θ
    pub t: f64,        // T
}

impl BiophysicalState {
    /// Create a new state.
    pub fn new(e: f64, m_prot: f64, s_bio: f64, theta: f64, t: f64) -> Self {
        Self { e, m_prot, s_bio, theta, t }
    }

    /// Return the state as an array for vector operations.
    pub fn as_array(&self) -> [f64; 5] {
        [self.e, self.m_prot, self.s_bio, self.theta, self.t]
    }

    /// Euclidean distance to another state (normalized coordinates assumed).
    pub fn distance(&self, other: &Self) -> f64 {
        self.as_array()
            .iter()
            .zip(other.as_array().iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Metadata about an active stimulus at the time a state was recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimulusMetadata {
    /// Unique identifier for the stimulus protocol (e.g., UUID).
    pub stimulus_id: String,
    /// Human-readable name of the stimulus.
    pub stimulus_name: String,
    /// Optional: the specific audio parameters used (if audio).
    pub audio_params: Option<AudioParams>,
}

/// Audio-specific parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    pub carrier_hz: f64,
    pub beat_hz: f64,
    pub amplitude: f64,
}

/// A biophysical state with optional stimulus metadata.
#[derive(Debug, Clone)]
pub struct TaggedState {
    pub state: BiophysicalState,
    pub timestamp: Instant,
    pub stimulus: Option<StimulusMetadata>,
}

/// A unique identifier for a micro-polytope.
pub type PolytopeId = uuid::Uuid;

/// A micro‑polytope: a compact summary of a cluster of nearby states.
///
/// Stores weighted sufficient statistics for online updates and decay.
/// Also accumulates craving associations and stimulus statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroPolytope {
    pub id: PolytopeId,
    /// Weighted number of points (sum of weights, affected by decay).
    pub weight: f64,
    /// Weighted linear sum of coordinates (for centroid).
    pub linear_sum: [f64; 5],
    /// Weighted sum of squares (for radius / spread).
    pub sq_sum: [f64; 5],
    /// Timestamp of last update (used for decay calculations).
    pub last_update: Instant,
    /// Craving statistics (optional).
    pub craving_sum: f64,
    pub craving_count: f64,
    /// Stimulus statistics: map from stimulus_id to count of points in this polytope
    /// that occurred under that stimulus.
    pub stimulus_counts: HashMap<String, u64>,
    /// Also store total points per stimulus (maybe for correlation later).
}

impl MicroPolytope {
    /// Create a new polytope from a single tagged point.
    pub fn from_point(point: &TaggedState) -> Self {
        let arr = point.state.as_array();
        let mut counts = HashMap::new();
        if let Some(stim) = &point.stimulus {
            counts.insert(stim.stimulus_id.clone(), 1);
        }
        Self {
            id: uuid::Uuid::new_v4(),
            weight: 1.0,
            linear_sum: arr,
            sq_sum: arr.map(|x| x * x),
            last_update: point.timestamp,
            craving_sum: 0.0,
            craving_count: 0.0,
            stimulus_counts: counts,
        }
    }

    /// Centroid of the polytope (weighted average).
    pub fn centroid(&self) -> BiophysicalState {
        let inv = 1.0 / self.weight;
        BiophysicalState::new(
            self.linear_sum[0] * inv,
            self.linear_sum[1] * inv,
            self.linear_sum[2] * inv,
            self.linear_sum[3] * inv,
            self.linear_sum[4] * inv,
        )
    }

    /// Radius as root‑mean‑square deviation from centroid.
    pub fn radius(&self) -> f64 {
        let centroid = self.centroid();
        let variance: f64 = (0..5)
            .map(|i| (self.sq_sum[i] / self.weight) - (centroid.as_array()[i]).powi(2))
            .sum();
        variance.sqrt()
    }

    /// Apply exponential decay to all summary statistics.
    /// `factor` should be in (0,1], typically `decay_rate.powf(dt)`.
    pub fn decay(&mut self, factor: f64) {
        self.weight *= factor;
        for i in 0..5 {
            self.linear_sum[i] *= factor;
            self.sq_sum[i] *= factor;
        }
        // Craving stats are not decayed; they are event counters, not time-series.
        // Stimulus counts are also not decayed; they are cumulative.
    }

    /// Incorporate a new tagged point into this polytope, applying time decay first.
    pub fn update(&mut self, point: &TaggedState, decay_rate: f64) {
        let dt = point.timestamp.duration_since(self.last_update).as_secs_f64();
        if dt > 0.0 {
            self.decay(decay_rate.powf(dt));
        }
        let arr = point.state.as_array();
        self.weight += 1.0;
        for i in 0..5 {
            self.linear_sum[i] += arr[i];
            self.sq_sum[i] += arr[i] * arr[i];
        }
        self.last_update = point.timestamp;

        // Update stimulus counts
        if let Some(stim) = &point.stimulus {
            *self.stimulus_counts.entry(stim.stimulus_id.clone()).or_insert(0) += 1;
        }
    }

    /// Associate a craving event with this polytope.
    pub fn add_craving(&mut self, intensity: f64) {
        self.craving_count += 1.0;
        self.craving_sum += intensity;
    }

    /// Average craving intensity when this polytope was active (if any).
    pub fn avg_craving(&self) -> Option<f64> {
        if self.craving_count > 0.0 {
            Some(self.craving_sum / self.craving_count)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_centroid_and_radius() {
        let now = Instant::now();
        let p1 = TaggedState {
            state: BiophysicalState::new(0.1, 0.2, 0.3, 0.4, 0.5),
            timestamp: now,
            stimulus: None,
        };
        let p2 = TaggedState {
            state: BiophysicalState::new(0.2, 0.3, 0.4, 0.5, 0.6),
            timestamp: now + Duration::from_secs(1),
            stimulus: None,
        };
        let mut poly = MicroPolytope::from_point(&p1);
        poly.update(&p2, 0.99);

        let centroid = poly.centroid();
        assert!((centroid.e - 0.15).abs() < 1e-6);
        assert!((centroid.m_prot - 0.25).abs() < 1e-6);

        let radius = poly.radius();
        assert!(radius > 0.0);
    }

    #[test]
    fn test_stimulus_counts() {
        let now = Instant::now();
        let stim1 = StimulusMetadata {
            stimulus_id: "abc".into(),
            stimulus_name: "test".into(),
            audio_params: None,
        };
        let p1 = TaggedState {
            state: BiophysicalState::new(0.1, 0.2, 0.3, 0.4, 0.5),
            timestamp: now,
            stimulus: Some(stim1),
        };
        let poly = MicroPolytope::from_point(&p1);
        assert_eq!(poly.stimulus_counts.get("abc"), Some(&1));
    }
}
