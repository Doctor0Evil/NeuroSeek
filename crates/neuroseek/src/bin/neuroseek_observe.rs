//! Observation‑only binary for NeuroSeek.
//!
//! Reads biophysical states from a JSON file (written by the nicotine safety stack),
//! feeds them into the clustering engine, and periodically logs or saves the
//! evolving polytope map. No device control, no neural data access.
//!
//! Usage:
//!   neuroseek_observe --state-file /path/to/state.json [--poll-ms 1000]

use clap::Parser;
use neuroseek::clustering::{ClustererConfig, PolytopeClusterer};
use neuroseek::ingest::episode_metrics::{IngesterConfig, MetricsIngester};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[clap(author, version, about = "NeuroSeek observation mode")]
struct Args {
    /// Path to JSON file containing the current BiophysicalState.
    #[clap(short, long)]
    state_file: PathBuf,

    /// Polling interval in milliseconds.
    #[clap(long, default_value_t = 1000)]
    poll_ms: u64,

    /// Optional directory to write polytope snapshots (JSON).
    #[clap(short, long)]
    snapshot_dir: Option<PathBuf>,

    /// Snapshot interval in seconds (0 to disable).
    #[clap(long, default_value_t = 60)]
    snapshot_interval_secs: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    // Clusterer configuration (tune as needed).
    let clusterer_config = ClustererConfig {
        radius_factor: 1.8,
        max_radius: 0.3,
        decay_rate: 0.995,
        merge_threshold: 0.15,
        min_weight: 3.0,
    };
    let maintenance_interval = Duration::from_secs(60);
    let clusterer = Arc::new(Mutex::new(PolytopeClusterer::new(
        clusterer_config,
        maintenance_interval,
    )));

    // Ingester.
    let ingester_config = IngesterConfig {
        state_file: args.state_file,
        poll_interval: Duration::from_millis(args.poll_ms),
    };
    let ingester = MetricsIngester::new(clusterer.clone(), ingester_config);

    // Optional snapshot writer.
    if let Some(dir) = args.snapshot_dir {
        tokio::fs::create_dir_all(&dir).await?;
        let snapshot_interval = Duration::from_secs(args.snapshot_interval_secs);
        let clusterer_for_snapshots = clusterer.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(snapshot_interval);
            loop {
                interval.tick().await;
                let polytopes = clusterer_for_snapshots.lock().await.polytopes().clone();
                let snapshot = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "polytopes": polytopes,
                });
                let filename = dir.join(format!("polytopes_{}.json", chrono::Utc::now().timestamp()));
                if let Err(e) = tokio::fs::write(&filename, serde_json::to_string_pretty(&snapshot)?).await {
                    eprintln!("Failed to write snapshot: {}", e);
                } else {
                    info!("Wrote snapshot to {}", filename.display());
                }
            }
        });
    }

    // Run the ingester (forever).
    ingester.run().await;
}
