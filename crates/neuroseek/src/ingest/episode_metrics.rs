//! Ingests biophysical state from a metrics source and feeds it into the clusterer.
//!
//! The `MetricsIngester` periodically reads the current state (e.g., from a JSON file
//! written by the nicotine safety stack) and inserts it into the `PolytopeClusterer`.
//! All operations are read‑only and use only outer‑domain signals.

use crate::clustering::PolytopeClusterer;
use crate::model::BiophysicalState;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;
use tracing::{error, info, warn};

/// Configuration for the metrics ingester.
#[derive(Debug, Clone)]
pub struct IngesterConfig {
    /// Path to a JSON file containing the current `BiophysicalState`.
    /// The file should be updated atomically (e.g., write to temp then rename).
    pub state_file: PathBuf,
    /// Polling interval.
    pub poll_interval: Duration,
}

/// Reads the current biophysical state from a JSON file.
async fn read_state_from_file(path: &Path) -> Option<BiophysicalState> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<BiophysicalState>(&contents) {
            Ok(state) => Some(state),
            Err(e) => {
                error!("Failed to parse state file {}: {}", path.display(), e);
                None
            }
        },
        Err(e) => {
            // It's normal for the file not to exist yet.
            if e.kind() != std::io::ErrorKind::NotFound {
                error!("Failed to read state file {}: {}", path.display(), e);
            }
            None
        }
    }
}

/// Ingests metrics by polling a file and updating the clusterer.
pub struct MetricsIngester {
    clusterer: Arc<Mutex<PolytopeClusterer>>,
    config: IngesterConfig,
}

impl MetricsIngester {
    /// Create a new ingester that will update the given clusterer.
    pub fn new(clusterer: Arc<Mutex<PolytopeClusterer>>, config: IngesterConfig) -> Self {
        Self { clusterer, config }
    }

    /// Run the ingestion loop (blocks forever).
    pub async fn run(&self) -> ! {
        let mut interval = time::interval(self.config.poll_interval);
        loop {
            interval.tick().await;
            if let Some(state) = read_state_from_file(&self.config.state_file).await {
                let mut clusterer = self.clusterer.lock().await;
                clusterer.insert_point(state, Instant::now());
                info!("Inserted state: {:?}", state);
            } else {
                warn!("No valid state available");
            }
        }
    }
}
