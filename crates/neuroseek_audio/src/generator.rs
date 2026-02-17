//! Generate WAV files from a fixed audio state or a trajectory within a polytope.

use crate::audio_nanopolytope::{AudioNanopolytope, AudioState};
use hound::{WavWriter, SampleFormat};
use std::f64::consts::PI;

/// Generate a WAV file from a fixed audio state (constant tones).
pub fn generate_wav_from_fixed(
    state: &AudioState,
    sample_rate: u32,
    output_path: &str,
) -> anyhow::Result<()> {
    let num_samples = (sample_rate as f64 * state.session_duration_sec) as usize;
    let dt = 1.0 / sample_rate as f64;

    let left_freq = state.carrier_hz;
    let right_freq = state.carrier_hz + state.beat_hz;
    let amp = state.amplitude.clamp(0.0, 1.0);

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(output_path, spec)?;

    for i in 0..num_samples {
        let t = i as f64 * dt;
        // Simple fade in/out (1 second each if duration > 2)
        let fade = if state.session_duration_sec > 2.0 {
            if t < 1.0 {
                t
            } else if t > state.session_duration_sec - 1.0 {
                state.session_duration_sec - t
            } else {
                1.0
            }
        } else {
            1.0
        };
        let sample_amp = amp * fade;

        let left = (sample_amp * (2.0 * PI * left_freq * t).sin()) as f32;
        let right = (sample_amp * (2.0 * PI * right_freq * t).sin()) as f32;

        writer.write_sample((left * i16::MAX as f32) as i16)?;
        writer.write_sample((right * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;
    Ok(())
}

/// Generate a WAV file by moving smoothly between two safe vertices of a polytope.
/// The trajectory is linear in audio space.
pub fn generate_wav_from_polytope(
    polytope: &AudioNanopolytope,
    start: &AudioState,
    end: &AudioState,
    sample_rate: u32,
    output_path: &str,
) -> anyhow::Result<()> {
    // Ensure both start and end are inside the polytope.
    if !polytope.contains(start) || !polytope.contains(end) {
        anyhow::bail!("Start or end point not inside the safe polytope");
    }

    let duration = end.session_duration_sec; // we use the end's duration as total length
    let num_samples = (sample_rate as f64 * duration) as usize;
    let dt = 1.0 / sample_rate as f64;

    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(output_path, spec)?;

    for i in 0..num_samples {
        let t = i as f64 * dt;
        let alpha = t / duration; // linear interpolation factor

        // Interpolate audio parameters
        let amp = start.amplitude + alpha * (end.amplitude - start.amplitude);
        let carrier = start.carrier_hz + alpha * (end.carrier_hz - start.carrier_hz);
        let beat = start.beat_hz + alpha * (end.beat_hz - start.beat_hz);

        // Fade in/out (1 second each)
        let fade = if duration > 2.0 {
            if t < 1.0 {
                t
            } else if t > duration - 1.0 {
                duration - t
            } else {
                1.0
            }
        } else {
            1.0
        };
        let sample_amp = amp.clamp(0.0, 1.0) * fade;

        let left = (sample_amp * (2.0 * PI * carrier * t).sin()) as f32;
        let right = (sample_amp * (2.0 * PI * (carrier + beat) * t).sin()) as f32;

        writer.write_sample((left * i16::MAX as f32) as i16)?;
        writer.write_sample((right * i16::MAX as f32) as i16)?;
    }
    writer.finalize()?;
    Ok(())
}
