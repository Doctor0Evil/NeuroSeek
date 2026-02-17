pub mod biophysics;      // (we need to create this if not exists)
pub mod clustering;
pub mod compiler;
pub mod governance;      // (we need to create this)
pub mod ingest;
pub mod model;
pub mod neurorights;     // (we need to create this)
pub mod stimulus;

// Re-export key types from other crates for convenience.
pub use neuroseek_audio;
