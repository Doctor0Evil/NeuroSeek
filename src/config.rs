use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinauralConfig {
    /// Name of the dose (e.g., "Nicotine", "Relaxation")
    pub name: String,
    /// Base carrier frequency in Hz (e.g., 200.0)
    pub carrier_hz: f64,
    /// Target beat frequency in Hz (difference between left and right)
    pub beat_hz: f64,
    /// Duration in seconds
    pub duration_sec: f64,
    /// Amplitude (0.0 to 1.0, will be clamped)
    pub amplitude: f64,
    /// Optional: ramp time (seconds) for fade in/out
    pub fade_sec: Option<f64>,
    /// Optional: background noise level (0.0 to 1.0)
    pub noise_level: Option<f64>,
}
