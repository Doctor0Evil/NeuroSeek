pub fn is_action_safe(protocol: &BiofieldProtocol, current_state: &BiophysicalState, corridor: &BiophysicalCorridor) -> bool {
    let predicted_trajectory = simulate_protocol_effect(protocol, current_state);
    predicted_trajectory.iter().all(|state| corridor.contains(state))
}

// Placeholder simulation – replace with data-driven model in Phase 3
fn simulate_protocol_effect(protocol: &BiofieldProtocol, start: &BiophysicalState) -> Vec<BiophysicalState> {
    // Very simple: linearly interpolate to a hypothetical end state
    let end_state = BiophysicalState {
        energy: start.energy + protocol.energy_delta,
        protein_marker: start.protein_marker + protocol.protein_delta,
        // ...
    };
    // Generate intermediate points (e.g., 10 steps)
    (0..=10).map(|i| {
        let t = i as f64 / 10.0;
        start.interpolate(&end_state, t)
    }).collect()
}