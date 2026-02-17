pub enum CompilationResult {
    Approved { token: uuid::Uuid, valid_until: SystemTime },
    Rejected { reasons: Vec<String> },
}

pub fn compile_protocol(
    protocol: &BiofieldProtocol,
    current_state: &BiophysicalState,
    corridor: &BiophysicalCorridor,
    rights: &NeurorightsConstraints,
    hardware_limits: &HardwareCapabilities, // e.g., max amplitude from emitter
) -> CompilationResult {
    let mut reasons = vec![];

    // 1. Hardware limit check
    if protocol.amplitude_ua > hardware_limits.max_amplitude_ua {
        reasons.push(format!("Amplitude {} uA exceeds hardware limit {}", protocol.amplitude_ua, hardware_limits.max_amplitude_ua));
    }

    // 2. Neurorights checks
    if !rights.prohibit_thought_decoding {
        reasons.push("Thought decoding must be prohibited".into());
    }
    // ... check other rights

    // 3. Biophysical safety simulation
    if !is_action_safe(protocol, current_state, corridor) {
        reasons.push("Protocol would violate biophysical corridors".into());
    }

    if reasons.is_empty() {
        // Generate a single-use token with expiry
        CompilationResult::Approved {
            token: uuid::Uuid::new_v4(),
            valid_until: SystemTime::now() + Duration::from_secs(60), // token valid for 1 minute
        }
    } else {
        CompilationResult::Rejected { reasons }
    }
}