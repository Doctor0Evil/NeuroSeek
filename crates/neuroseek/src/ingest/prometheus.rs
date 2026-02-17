//! Ingests biophysical state by directly querying a Prometheus server.
//!
//! The `PrometheusIngester` periodically queries Prometheus instant queries
//! for each of the five biophysical dimensions, constructs a `BiophysicalState`,
//! and feeds it into the clusterer. All operations are read‑only and use only
//! outer‑domain signals.

use crate::clustering::PolytopeClusterer;
use crate::model::BiophysicalState;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;
use tracing::{error, info, warn};

/// Configuration for the Prometheus ingester.
#[derive(Debug, Clone)]
pub struct PrometheusConfig {
    /// Prometheus server URL (e.g., "http://localhost:9090").
    pub server_url: String,
    /// Prometheus instant query for the energy dimension (E).
    pub query_e: String,
    /// Prometheus instant query for the protein burden (M_prot).
    pub query_m_prot: String,
    /// Prometheus instant query for the biostress index (S_bio).
    pub query_s_bio: String,
    /// Prometheus instant query for the brain state proxy (theta).
    pub query_theta: String,
    /// Prometheus instant query for the temperature deviation (T).
    pub query_t: String,
    /// Polling interval.
    pub poll_interval: Duration,
}

/// Ingests metrics by querying Prometheus and updating the clusterer.
pub struct PrometheusIngester {
    clusterer: Arc<Mutex<PolytopeClusterer>>,
    config: PrometheusConfig,
    client: Client,
}

impl PrometheusIngester {
    /// Create a new Prometheus ingester.
    pub fn new(clusterer: Arc<Mutex<PolytopeClusterer>>, config: PrometheusConfig) -> Self {
        Self {
            clusterer,
            config,
            client: Client::new(),
        }
    }

    /// Perform a single instant query and return the value as f64.
    async fn query_metric(&self, query: &str) -> Option<f64> {
        let url = format!(
            "{}/api/v1/query?query={}",
            self.config.server_url,
            urlencoding::encode(query)
        );
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    error!("Prometheus query failed with status: {}", resp.status());
                    return None;
                }
                match resp.json::<Value>().await {
                    Ok(json) => {
                        // Expected Prometheus response structure:
                        // {"status":"success","data":{"resultType":"vector","result":[{"metric":{},"value":[timestamp,"value"]}]}}
                        if let Some(result) = json["data"]["result"].as_array() {
                            if let Some(first) = result.first() {
                                if let Some(value_str) = first["value"].as_array().and_then(|v| v.get(1)).and_then(|v| v.as_str()) {
                                    if let Ok(val) = value_str.parse::<f64>() {
                                        return Some(val);
                                    }
                                }
                            }
                        }
                        error!("Unexpected Prometheus response format: {}", json);
                        None
                    }
                    Err(e) => {
                        error!("Failed to parse Prometheus JSON response: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to query Prometheus: {}", e);
                None
            }
        }
    }

    /// Run the ingestion loop (blocks forever).
    pub async fn run(&self) -> ! {
        let mut interval = time::interval(self.config.poll_interval);
        loop {
            interval.tick().await;
            // Query all five dimensions concurrently.
            let (e, m_prot, s_bio, theta, t) = tokio::join!(
                self.query_metric(&self.config.query_e),
                self.query_metric(&self.config.query_m_prot),
                self.query_metric(&self.config.query_s_bio),
                self.query_metric(&self.config.query_theta),
                self.query_metric(&self.config.query_t),
            );

            if let (Some(e), Some(m_prot), Some(s_bio), Some(theta), Some(t)) = (e, m_prot, s_bio, theta, t) {
                let state = BiophysicalState::new(e, m_prot, s_bio, theta, t);
                let mut clusterer = self.clusterer.lock().await;
                clusterer.insert_point(state, Instant::now());
                info!("Inserted state from Prometheus: {:?}", state);
            } else {
                warn!("Failed to retrieve all metrics from Prometheus");
            }
        }
    }
}
