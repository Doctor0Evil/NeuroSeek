//! Safety compiler: validates stimulus protocols against biophysical corridors and neurorights.
//!
//! The compiler returns a token if the protocol is safe; otherwise returns rejection reasons.

use crate::biophysics::{BiophysicalCorridor, BiophysicalState};
use crate::neurorights::NeurorightsConstraints;
use neuroseek_audio::audio_nanopolytope::{AudioNanopolytope, AudioState};
use neuroseek_audio::config::Protocol as AudioProtocol;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// A stimulus can be audio or other modalities (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stimulus {
    Audio(AudioStimulus),
}

/// Audio stimulus: either a fixed point or a polytope to explore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStimulus {
    pub protocol: AudioProtocol,
    pub name: String,
    pub description: Option<String>,
}

/// Result of compilation.
#[derive(Debug, Clone)]
pub enum CompilationResult {
    Approved {
        token: Uuid,
        valid_until: SystemTime,
        stimulus: Stimulus,
    },
    Rejected {
        reasons: Vec<String>,
    },
}

/// Hardware capabilities (e.g., max amplitude, sample rate).
#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub max_amplitude: f64,
    pub min_frequency: f64,
    pub max_frequency: f64,
    pub max_session_duration_sec: f64,
    pub max_daily_duty: f64,
}

/// Compiler configuration.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub hardware: HardwareCapabilities,
    pub biophysical_corridor: BiophysicalCorridor,
    pub neurorights: NeurorightsConstraints,
    pub token_validity_duration: Duration,
}

/// The safety compiler.
pub struct Compiler {
    config: CompilerConfig,
}

impl Compiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a stimulus protocol. Returns a token if safe.
    pub fn compile(&self, stimulus: Stimulus) -> CompilationResult {
        match stimulus {
            Stimulus::Audio(audio) => self.compile_audio(audio),
        }
    }

    fn compile_audio(&self, audio: AudioStimulus) -> CompilationResult {
        let mut reasons = Vec::new();

        match &audio.protocol {
            AudioProtocol::Fixed(state) => {
                if let Err(e) = self.check_audio_state(state) {
                    reasons.push(e);
                }
            }
            AudioProtocol::Polytope(poly) => {
                // For a polytope, we need to check that all vertices (or at least a representative set) are safe.
                // For now, we check a few sample points: the center and extremes along each axis.
                // In production, you'd want to ensure the entire polytope is inside the safe region.
                if let Some(center) = poly.center() {
                    if let Err(e) = self.check_audio_state(&center) {
                        reasons.push(format!("Polytope center unsafe: {}", e));
                    }
                }
                // Also check a random point (if sampling works)
                if let Some(random) = poly.sample_random() {
                    if let Err(e) = self.check_audio_state(&random) {
                        reasons.push(format!("Polytope random point unsafe: {}", e));
                    }
                } else {
                    reasons.push("Cannot sample polytope (maybe unbounded)".into());
                }
            }
        }

        // Check neurorights
        if !self.config.neurorights.prohibit_thought_decoding {
            reasons.push("Thought decoding must be prohibited".into());
        }
        // Additional neurorights checks can be added here.

        if reasons.is_empty() {
            let token = Uuid::new_v4();
            let valid_until = SystemTime::now() + self.config.token_validity_duration;
            CompilationResult::Approved {
                token,
                valid_until,
                stimulus: Stimulus::Audio(audio),
            }
        } else {
            CompilationResult::Rejected { reasons }
        }
    }

    /// Check a single audio state against hardware limits and biophysical projection.
    fn check_audio_state(&self, state: &AudioState) -> Result<(), String> {
        // Hardware limits
        if state.amplitude > self.config.hardware.max_amplitude {
            return Err(format!("Amplitude {} exceeds hardware max {}", state.amplitude, self.config.hardware.max_amplitude));
        }
        if state.carrier_hz < self.config.hardware.min_frequency || state.carrier_hz > self.config.hardware.max_frequency {
            return Err(format!("Carrier frequency {} out of hardware range", state.carrier_hz));
        }
        if state.beat_hz < 0.0 || state.beat_hz > 30.0 { // typical safe range
            return Err(format!("Beat frequency {} out of safe range (0-30 Hz)", state.beat_hz));
        }
        if state.session_duration_sec > self.config.hardware.max_session_duration_sec {
            return Err(format!("Session duration {} exceeds max {}", state.session_duration_sec, self.config.hardware.max_session_duration_sec));
        }
        if state.duty > self.config.hardware.max_daily_duty {
            return Err(format!("Duty {} exceeds max daily duty {}", state.duty, self.config.hardware.max_daily_duty));
        }

        // Map to biophysical state if transfer function available.
        // For now, we assume a simple linear transfer (placeholder). In practice, you'd use the polytope's transfer.
        // We'll just check that the projected biophysical state stays within corridor.
        let projected = self.project_audio_to_biophysical(state);
        if !self.config.biophysical_corridor.contains(&projected) {
            return Err("Projected biophysical state outside safe corridor".into());
        }

        Ok(())
    }

    /// Placeholder projection function. In reality, use the transfer matrix from the polytope.
    fn project_audio_to_biophysical(&self, state: &AudioState) -> BiophysicalState {
        // Example: assume amplitude affects energy, beat affects stress, etc.
        BiophysicalState {
            e: 0.2 * state.amplitude,
            m_prot: 0.0,
            s_bio: 0.1 * state.beat_hz / 10.0,
            theta: 0.5 * (state.carrier_hz / 500.0),
            t: 0.0,
        }
    }
}
