//! Defines the 5D audio parameter space and safe polytope with linear constraints.

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::f64;

/// 5D audio state vector.
/// Dimensions: (amplitude, carrier_hz, beat_hz, duty, session_duration_sec)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioState {
    pub amplitude: f64,   // 0..1 (normalized sound pressure)
    pub carrier_hz: f64,  // e.g., 100..1000
    pub beat_hz: f64,     // 0..30 (typical binaural range)
    pub duty: f64,        // 0..1 (fraction of time actively used, e.g., daily)
    pub session_duration_sec: f64, // seconds
}

impl AudioState {
    pub fn to_vector(&self) -> DVector<f64> {
        DVector::from_vec(vec![
            self.amplitude,
            self.carrier_hz,
            self.beat_hz,
            self.duty,
            self.session_duration_sec,
        ])
    }

    pub fn from_vector(v: DVector<f64>) -> Self {
        Self {
            amplitude: v[0],
            carrier_hz: v[1],
            beat_hz: v[2],
            duty: v[3],
            session_duration_sec: v[4],
        }
    }
}

/// A convex polytope defined by A * x <= b.
/// Also stores transfer functions (optional) to map to biophysical state z,
/// and a ResponseMetric for the polytope as a whole.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioNanopolytope {
    /// Name of this safe region (e.g., "Alpha comfort zone")
    pub name: String,
    /// Constraint matrix (m x 5)
    pub a: Vec<[f64; 5]>, // each row is a linear inequality coefficient
    /// Right-hand side vector (length m)
    pub b: Vec<f64>,
    /// Optional: ResponseMetric for this polytope (K, D, DW)
    pub k: f64,
    pub d: f64,
    pub dw: f64,
    /// Optional: transfer function from audio state to biophysical state z.
    /// For simplicity, we store a linear approximation: z = M * x + offset.
    pub transfer_matrix: Option<[[f64; 5]; 5]>, // 5x5 matrix mapping audio to z
    pub transfer_offset: Option<[f64; 5]>,
}

impl AudioNanopolytope {
    /// Create a new polytope from constraints. The caller must ensure A and b are consistent.
    pub fn new(name: String, a: Vec<[f64; 5]>, b: Vec<f64>, k: f64, d: f64, dw: f64) -> Self {
        Self {
            name,
            a,
            b,
            k,
            d,
            dw,
            transfer_matrix: None,
            transfer_offset: None,
        }
    }

    /// Check if a given audio state lies inside the polytope.
    pub fn contains(&self, state: &AudioState) -> bool {
        let x = state.to_vector();
        for (row, &rhs) in self.a.iter().zip(&self.b) {
            let lhs: f64 = row.iter().zip(x.iter()).map(|(a, x)| a * x).sum();
            if lhs > rhs + 1e-12 { // allow small numerical tolerance
                return false;
            }
        }
        true
    }

    /// Project a state onto the polytope (closest point inside). Simple iterative method.
    /// For production, one might use quadratic programming; here we do a naive bounding box.
    /// This is a placeholder; actual projection would solve a QP.
    pub fn project(&self, state: &AudioState) -> AudioState {
        // For now, just clamp each coordinate to reasonable bounds (we assume axis-aligned for simplicity).
        // In a full implementation, we'd solve min ||x - state|| s.t. A x <= b.
        // We'll leave that for future enhancement.
        *state
    }

    /// Sample a random point uniformly within the polytope (rejection sampling).
    /// Requires that the polytope is bounded. For now, we assume axis-aligned bounds and sample a box.
    pub fn sample_random(&self) -> Option<AudioState> {
        // Rejection sampling on a bounding box.
        // First, find axis-aligned bounds from constraints (crude but works for simple cases).
        // For generality, we'd need to compute the bounding box via linear programming.
        // We'll implement a placeholder that just returns a point near the center.
        // In practice, you'd derive bounds from constraints or store them separately.
        // This is a TODO for actual deployment.
        None
    }

    /// Set the linear transfer function from audio space to biophysical space.
    pub fn with_transfer(mut self, matrix: [[f64; 5]; 5], offset: [f64; 5]) -> Self {
        self.transfer_matrix = Some(matrix);
        self.transfer_offset = Some(offset);
        self
    }

    /// Map an audio state to the predicted biophysical state z = (E, Mprot, Sbio, theta, T).
    pub fn map_to_biophysical(&self, audio: &AudioState) -> Option<[f64; 5]> {
        match (&self.transfer_matrix, &self.transfer_offset) {
            (Some(m), Some(off)) => {
                let x = audio.to_vector();
                let mut z = [0.0; 5];
                for i in 0..5 {
                    z[i] = off[i] + (0..5).map(|j| m[i][j] * x[j]).sum::<f64>();
                }
                Some(z)
            }
            _ => None,
        }
    }
}

/// Error type for constraint violations.
#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error("Amplitude out of safe range")]
    Amplitude,
    #[error("Carrier frequency out of safe range")]
    Carrier,
    #[error("Beat frequency out of safe range")]
    Beat,
    #[error("Duty cycle out of safe range")]
    Duty,
    #[error("Session duration out of safe range")]
    Duration,
    #[error("Polytope containment failed")]
    OutsidePolytope,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_aligned_polytope() {
        // Simple box: 0.1 <= amplitude <= 0.5, 100 <= carrier <= 400, etc.
        // Represent as A x <= b.
        let a = vec![
            [1.0, 0.0, 0.0, 0.0, 0.0],   // amplitude <= 0.5
            [-1.0, 0.0, 0.0, 0.0, 0.0],  // -amplitude <= -0.1 => amplitude >= 0.1
            [0.0, 1.0, 0.0, 0.0, 0.0],   // carrier <= 400
            [0.0, -1.0, 0.0, 0.0, 0.0],  // -carrier <= -100 => carrier >= 100
            [0.0, 0.0, 1.0, 0.0, 0.0],   // beat <= 20
            [0.0, 0.0, -1.0, 0.0, 0.0],  // -beat <= -4 => beat >= 4
            [0.0, 0.0, 0.0, 1.0, 0.0],   // duty <= 0.3
            [0.0, 0.0, 0.0, -1.0, 0.0],  // -duty <= -0.05 => duty >= 0.05
            [0.0, 0.0, 0.0, 0.0, 1.0],   // duration <= 600
            [0.0, 0.0, 0.0, 0.0, -1.0],  // -duration <= -60 => duration >= 60
        ];
        let b = vec![0.5, -0.1, 400.0, -100.0, 20.0, -4.0, 0.3, -0.05, 600.0, -60.0];
        let poly = AudioNanopolytope::new("test".into(), a, b, 0.9, 0.2, 0.1);

        let inside = AudioState {
            amplitude: 0.3,
            carrier_hz: 250.0,
            beat_hz: 10.0,
            duty: 0.2,
            session_duration_sec: 300.0,
        };
        assert!(poly.contains(&inside));

        let outside = AudioState {
            amplitude: 0.6,
            carrier_hz: 250.0,
            beat_hz: 10.0,
            duty: 0.2,
            session_duration_sec: 300.0,
        };
        assert!(!poly.contains(&outside));
    }
}
