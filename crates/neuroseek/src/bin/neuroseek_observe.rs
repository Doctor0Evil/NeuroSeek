//! Observation‑only binary for NeuroSeek.
//!
//! Reads biophysical states either from a JSON file or by directly querying Prometheus,
//! feeds them into the clustering engine, and periodically logs or saves the
//! evolving polytope map. No device control, no neural data access.
//!
//! Examples:
//!   neuroseek_observe --source file --state-file /tmp/state.json
//!   neuroseek_observe --source prometheus --prometheus-url http://localhost:9090

use clap::{Parser, Subcommand};
use neuroseek::clustering::{ClustererConfig, PolytopeClusterer};
use neuroseek::ingest::episode_metrics::{IngesterConfig, MetricsIngester};
use neuroseek::ingest::prometheus::{PrometheusConfig, PrometheusIngester};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[clap(author, version, about = "NeuroSeek observation mode")]
struct Args {
    /// Source of biophysical state data.
    #[clap(subcommand)]
    source: Source,

    /// Optional directory to write polytope snapshots (JSON).
    #[clap(short, long)]
    snapshot_dir: Option<PathBuf>,

    /// Snapshot interval in seconds (0 to disable).
    #[clap(long, default_value_t = 60)]
    snapshot_interval_secs: u64,
}

#[derive(Subcommand, Debug)]
enum Source {
    /// Read from a JSON file (written by the nicotine safety stack).
    File {
        /// Path to JSON file containing the current BiophysicalState.
        #[clap(short, long)]
        state_file: PathBuf,

        /// Polling interval in milliseconds.
        #[clap(long, default_value_t = 1000)]
        poll_ms: u64,
    },
    /// Query Prometheus directly.
    Prometheus {
        /// Prometheus server URL.
        #[clap(short, long)]
        prometheus_url: String,

        /// Prometheus query for energy (E).
        #[clap(long, default_value = "avg_over_time(heart_rate_variability[1m])")]
        query_e: String,

        /// Prometheus query for protein burden (M_prot).
        #[clap(long, default_value = "protein_oxidation_level")]
        query_m_prot: String,

        /// Prometheus query for biostress (S_bio).
        #[clap(long, default_value = "stress_index")]
        query_s_bio: String,

        /// Prometheus query for theta proxy.
        #[clap(long, default_value = "brain_theta_delta_ratio")]
        query_theta: String,

        /// Prometheus query for temperature deviation (T).
        #[clap(long, default_value = "skin_temperature_deviation")]
        query_t: String,

        /// Prometheus query for active stimulus (returns a metric with labels id, name, etc.)
        #[clap(long, default_value = "active_stimulus")]
        query_stimulus: String,

        /// Polling interval in milliseconds.
        #[clap(long, default_value_t = 1000)]
        poll_ms: u64,
    },
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

    // Spawn snapshot writer if requested.
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

    // Run the appropriate ingester.
    match args.source {
        Source::File { state_file, poll_ms } => {
            let ingester_config = IngesterConfig {
                state_file,
                poll_interval: Duration::from_millis(poll_ms),
            };
            let ingester = MetricsIngester::new(clusterer, ingester_config);
            ingester.run().await;
        }
        Source::Prometheus {
            prometheus_url,
            query_e,
            query_m_prot,
            query_s_bio,
            query_theta,
            query_t,
            query_stimulus,
            poll_ms,
        } => {
            let prom_config = PrometheusConfig {
                server_url: prometheus_url,
                query_e,
                query_m_prot,
                query_s_bio,
                query_theta,
                query_t,
                query_active_stimulus: query_stimulus,
                query_stimulus_name: None, // we can extract name from labels
                poll_interval: Duration::from_millis(poll_ms),
            };
            let ingester = PrometheusIngester::new(clusterer, prom_config);
            ingester.run().await;
        }
    }
}
