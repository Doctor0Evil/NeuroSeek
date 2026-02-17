use hound::{WavWriter, SampleFormat};
use std::f64::consts::PI;

pub fn generate_binaural_wav(
    config: &BinauralConfig,
    sample_rate: u32,
    output_path: &str,
) -> anyhow::Result<()> {
    let num_samples = (sample_rate as f64 * config.duration_sec) as usize;
    let dt = 1.0 / sample_rate as f64;

    // Stereo: left = carrier, right = carrier + beat
    let left_freq = config.carrier_hz;
    let right_freq = config.carrier_hz + config.beat_hz;

    // Prepare WAV writer
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(output_path, spec)?;

    // Pre‑compute amplitude with optional fade
    for i in 0..num_samples {
        let t = i as f64 * dt;
        let amp = config.amplitude
            * fade_envelope(t, config.duration_sec, config.fade_sec.unwrap_or(0.0));

        let left_sample = (amp * (2.0 * PI * left_freq * t).sin()) as f32;
        let right_sample = (amp * (2.0 * PI * right_freq * t).sin()) as f32;

        // Convert to 16-bit integer
        let left_int = (left_sample * i16::MAX as f32) as i16;
        let right_int = (right_sample * i16::MAX as f32) as i16;

        writer.write_sample(left_int)?;
        writer.write_sample(right_int)?;
    }
    writer.finalize()?;
    Ok(())
}

fn fade_envelope(t: f64, total: f64, fade: f64) -> f64 {
    if fade <= 0.0 { return 1.0; }
    if t < fade {
        t / fade
    } else if t > total - fade {
        (total - t) / fade
    } else {
        1.0
    }
}
