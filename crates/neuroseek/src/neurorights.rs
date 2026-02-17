//! Neurorights constraints for the compiler.

#[derive(Debug, Clone)]
pub struct NeurorightsConstraints {
    pub prohibit_thought_decoding: bool,
    pub require_consent_before_application: bool,
    pub max_sessions_per_day: u32,
    pub max_cumulative_amplitude_per_session: f64,
}

impl Default for NeurorightsConstraints {
    fn default() -> Self {
        Self {
            prohibit_thought_decoding: true,
            require_consent_before_application: true,
            max_sessions_per_day: 4,
            max_cumulative_amplitude_per_session: 0.8,
        }
    }
}
