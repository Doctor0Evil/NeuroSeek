//! NeuroSeek Audio – Governed binaural beat generator integrated with the nicotine safety stack.
//!
//! Provides:
//! - `AudioNanopolytope`: a safe region in 5D audio parameter space.
//! - `AudioState`: a point in that space.
//! - Generator that produces stereo WAV files or real‑time streams from a polytope trajectory.
//! - Governance logging with ResponseMetric and nanoswarm telemetry.

pub mod audio_nanopolytope;
pub mod config;
pub mod generator;
pub mod governance;
pub mod transfer;

pub use audio_nanopolytope::{AudioNanopolytope, AudioState, ConstraintError};
pub use config::Protocol;
pub use generator::{generate_wav_from_polytope, generate_wav_from_fixed};
pub use governance::SessionLog;
