// A description of a biofield emission pattern
pub struct BiofieldProtocol {
    pub id: String,
    pub waveform_type: Waveform, // e.g., Sine, Pulsed, Noise
    pub frequency_hz: f64,
    pub amplitude_ua: f64,        // microamps – must stay below hardware limit
    pub duration_sec: u64,
    pub energy_delta: f64,        // estimated effect on your 5D coordinates
    pub protein_delta: f64,
    pub stress_delta: f64,
    pub theta_delta: f64,
    pub delta_temp_delta: f64,
}

// Immutable neurorights rules – these are checked at compile time
pub struct NeurorightsConstraints {
    pub prohibit_thought_decoding: bool,        // always true
    pub require_consent_before_application: bool, // always true
    pub max_sessions_per_day: u32,
    pub max_cumulative_amplitude_per_session: f64,
    // ... any other rights you encode
}