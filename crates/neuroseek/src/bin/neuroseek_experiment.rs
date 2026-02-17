//! Run a single audio experiment: compile, generate, log.
//!
//! Usage: neuroseek_experiment <protocol.json>

use clap::Parser;
use neuroseek::biophysics::{BiophysicalCorridor, BiophysicalState};
use neuroseek::compiler::{Compiler, CompilerConfig, HardwareCapabilities, Stimulus, AudioStimulus};
use neuroseek::neurorights::NeurorightsConstraints;
use neuroseek::stimulus::audio::{AudioOutputConfig, AudioStimulusExecutor, DummyTelemetryProvider};
use neuroseek_audio::config::Protocol as AudioProtocol;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
struct Args {
    /// Path to JSON file containing an AudioProtocol (fixed or polytope).
    protocol_file: PathBuf,
    /// Output directory for generated WAV files.
    #[arg(short, long, default_value = "./audio_output")]
    output_dir: PathBuf,
    /// Sample rate for generated audio.
    #[arg(long, default_value_t = 44100)]
    sample_rate: u32,
    /// Play audio after generation.
    #[arg(long)]
    play: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Read protocol
    let json = fs::read_to_string(&args.protocol_file)?;
    let audio_proto: AudioProtocol = serde_json::from_str(&json)?;

    // Create stimulus
    let stimulus = Stimulus::Audio(AudioStimulus {
        protocol: audio_proto,
        name: args.protocol_file.file_stem().unwrap().to_string_lossy().to_string(),
        description: None,
    });

    // Configure compiler
    let hardware = HardwareCapabilities {
        max_amplitude: 1.0,
        min_frequency: 20.0,
        max_frequency: 20000.0,
        max_session_duration_sec: 3600.0,
        max_daily_duty: 0.3,
    };
    let corridor = BiophysicalCorridor {
        e: (0.0, 0.8),
        m_prot: (0.0, 0.5),
        s_bio: (0.0, 0.7),
        theta: (0.0, 1.0),
        t: (-1.0, 1.0),
    };
    let neurorights = NeurorightsConstraints::default();
    let compiler_config = CompilerConfig {
        hardware,
        biophysical_corridor: corridor,
        neurorights,
        token_validity_duration: Duration::from_secs(300),
    };
    let compiler = Compiler::new(compiler_config);

    // Compile
    let result = compiler.compile(stimulus);
    match result {
        neuroseek::compiler::CompilationResult::Approved { token, valid_until, stimulus } => {
            println!("✅ Approved. Token: {}, valid until {:?}", token, valid_until);

            // Set up executor
            let audio_config = AudioOutputConfig {
                sample_rate: args.sample_rate,
                output_dir: args.output_dir,
                play_after_generate: args.play,
            };
            let telemetry = Arc::new(Mutex::new(DummyTelemetryProvider));
            let executor = AudioStimulusExecutor::new(audio_config)
                .with_telemetry_provider(telemetry);

            // Execute
            let (path, log) = executor.execute(stimulus.as_audio().unwrap()).await?;
            println!("Generated audio: {}", path.display());
            println!("Session log: {:#?}", log);
        }
        neuroseek::compiler::CompilationResult::Rejected { reasons } => {
            eprintln!("❌ Rejected:");
            for r in reasons {
                eprintln!("  - {}", r);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

// Helper to extract audio stimulus
impl Stimulus {
    fn as_audio(&self) -> Option<&AudioStimulus> {
        match self {
            Stimulus::Audio(a) => Some(a),
        }
    }
}
