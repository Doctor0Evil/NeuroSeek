//! Audio stimulus execution: generates sound and logs session data.

use crate::compiler::AudioStimulus;
use crate::governance::{NanoswarmTelemetry, SessionLog};
use neuroseek_audio::audio_nanopolytope::AudioState;
use neuroseek_audio::generator::{generate_wav_from_fixed, generate_wav_from_polytope};
use neuroseek_audio::governance::SessionLog as AudioSessionLog;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

/// Configuration for audio output.
#[derive(Debug, Clone)]
pub struct AudioOutputConfig {
    pub sample_rate: u32,
    pub output_dir: PathBuf,
    /// If true, also play the generated file using the system audio.
    pub play_after_generate: bool,
}

/// Executes an audio stimulus.
pub struct AudioStimulusExecutor {
    config: AudioOutputConfig,
    /// Optional: handle to a telemetry source to get nanoswarm data before/after.
    telemetry_source: Option<Arc<Mutex<dyn TelemetryProvider + Send>>>,
}

impl AudioStimulusExecutor {
    pub fn new(config: AudioOutputConfig) -> Self {
        Self {
            config,
            telemetry_source: None,
        }
    }

    /// Set a telemetry provider to capture nanoswarm data.
    pub fn with_telemetry_provider(mut self, provider: Arc<Mutex<dyn TelemetryProvider + Send>>) -> Self {
        self.telemetry_source = Some(provider);
        self
    }

    /// Execute an approved audio stimulus. Returns the path to the generated file and a session log.
    pub async fn execute(&self, stimulus: &AudioStimulus) -> anyhow::Result<(PathBuf, SessionLog)> {
        // Get telemetry before (if available)
        let telemetry_before = if let Some(ref provider) = self.telemetry_source {
            Some(provider.lock().await.get_telemetry().await?)
        } else {
            None
        };

        // Generate filename based on name and timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = self.config.output_dir.join(format!("{}_{}.wav", stimulus.name, timestamp));

        // Generate audio based on protocol type
        match &stimulus.protocol {
            neuroseek_audio::config::Protocol::Fixed(state) => {
                generate_wav_from_fixed(state, self.config.sample_rate, filename.to_str().unwrap())?;
            }
            neuroseek_audio::config::Protocol::Polytope(poly) => {
                // For polytope, we need to choose a trajectory. Here we just pick two points (start/end)
                // In a real implementation, you'd have a controller that picks vertices based on craving state.
                // For now, we pick the center as start and a slightly different point as end.
                let center = poly.center().ok_or_else(|| anyhow::anyhow!("Polytope has no center"))?;
                let mut end = center.clone();
                end.amplitude = (end.amplitude + 0.1).min(1.0);
                end.beat_hz = (end.beat_hz + 2.0).min(30.0);
                generate_wav_from_polytope(poly, &center, &end, self.config.sample_rate, filename.to_str().unwrap())?;
            }
        }

        // Play if requested
        if self.config.play_after_generate {
            if let Err(e) = self.play_audio(&filename).await {
                error!("Failed to play audio: {}", e);
            }
        }

        // Get telemetry after
        let telemetry_after = if let Some(ref provider) = self.telemetry_source {
            Some(provider.lock().await.get_telemetry().await?)
        } else {
            None
        };

        // Create session log
        let audio_log = match &stimulus.protocol {
            neuroseek_audio::config::Protocol::Fixed(state) => {
                // We don't have a polytope reference here, so we need to construct a minimal log.
                // For now, we create a dummy polytope with default K/D/DW.
                AudioSessionLog::new(
                    &neuroseek_audio::audio_nanopolytope::AudioNanopolytope::new(
                        stimulus.name.clone(),
                        vec![],
                        vec![],
                        0.9, 0.2, 0.1,
                    ),
                    *state,
                    telemetry_before.clone(),
                )
            }
            neuroseek_audio::config::Protocol::Polytope(poly) => {
                let state = poly.center().unwrap_or(AudioState {
                    amplitude: 0.0,
                    carrier_hz: 0.0,
                    beat_hz: 0.0,
                    duty: 0.0,
                    session_duration_sec: 0.0,
                });
                AudioSessionLog::new(poly, state, telemetry_before.clone())
            }
        };

        let session_log = SessionLog {
            audio_log,
            telemetry_after,
            stimulus_name: stimulus.name.clone(),
            // ... other fields
        };

        Ok((filename, session_log))
    }

    async fn play_audio(&self, path: &PathBuf) -> anyhow::Result<()> {
        // Use rodio or similar to play the file.
        // For now, we just log.
        info!("Would play audio from {}", path.display());
        Ok(())
    }
}

/// Trait for telemetry provider (e.g., from Prometheus or nanoswarm).
#[async_trait::async_trait]
pub trait TelemetryProvider {
    async fn get_telemetry(&mut self) -> anyhow::Result<NanoswarmTelemetry>;
}

/// A simple in-memory telemetry provider for testing.
pub struct DummyTelemetryProvider;

#[async_trait::async_trait]
impl TelemetryProvider for DummyTelemetryProvider {
    async fn get_telemetry(&mut self) -> anyhow::Result<NanoswarmTelemetry> {
        Ok(NanoswarmTelemetry {
            thermal_distance_index: 0.5,
            molecular_balance_index: 0.6,
            fatigue: 0.2,
            risk: 0.1,
            eco_impact: 0.05,
            nicotine_burden: 0.3,
        })
    }
}
