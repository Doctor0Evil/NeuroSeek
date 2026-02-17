// When a craving event occurs
fn on_craving_event(intensity: f64) {
    let current_state = get_current_state();
    if let Some(polytope) = find_polytope_containing(&current_state) {
        // Update that polytope's craving stats
        polytope.update_craving_correlation(intensity);
    }
}