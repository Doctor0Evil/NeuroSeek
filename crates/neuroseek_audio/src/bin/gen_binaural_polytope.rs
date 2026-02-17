//! CLI to generate a WAV file from a polytope JSON config.
//! Usage: gen_binaural_polytope <config.json> <output.wav>

use neuroseek_audio::config::Protocol;
use neuroseek_audio::generator::{generate_wav_from_fixed, generate_wav_from_polytope};
use neuroseek_audio::audio_nanopolytope::AudioState;
use std::fs;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <protocol.json> <output.wav>", args[0]);
        std::process::exit(1);
    }

    let json = fs::read_to_string(&args[1])?;
    let protocol: Protocol = serde_json::from_str(&json)?;

    match protocol {
        Protocol::Fixed(state) => {
            generate_wav_from_fixed(&state, 44100, &args[2])?;
            println!("Generated fixed audio at {}", args[2]);
        }
        Protocol::Polytope(poly) => {
            // For now, generate a simple trajectory: start at a random safe point, end at another.
            // In practice you'd define start/end based on craving state.
            // Here we just pick two opposite corners of the bounding box (if axis-aligned) as example.
            // We'll create a dummy start/end for demonstration.
            let start = AudioState {
                amplitude: 0.2,
                carrier_hz: 200.0,
                beat_hz: 5.0,
                duty: 0.1,
                session_duration_sec: 300.0,
            };
            let end = AudioState {
                amplitude: 0.4,
                carrier_hz: 300.0,
                beat_hz: 12.0,
                duty: 0.2,
                session_duration_sec: 300.0,
            };
            generate_wav_from_polytope(&poly, &start, &end, 44100, &args[2])?;
            println!("Generated polytope trajectory audio at {}", args[2]);
        }
    }

    Ok(())
}
