use neuroseek_audio::config::BinauralConfig;
use neuroseek_audio::generator::generate_binaural_wav;
use std::fs;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <config.json> <output.wav>", args[0]);
        std::process::exit(1);
    }
    let config_json = fs::read_to_string(&args[1])?;
    let config: BinauralConfig = serde_json::from_str(&config_json)?;
    generate_binaural_wav(&config, 44100, &args[2])?;
    println!("Generated {}", args[2]);
    Ok(())
}
