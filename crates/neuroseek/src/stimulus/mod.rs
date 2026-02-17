//! Stimulus execution: audio, and in the future other modalities.

pub mod audio;

// Re-export key types.
pub use audio::{AudioOutputConfig, AudioStimulusExecutor};
