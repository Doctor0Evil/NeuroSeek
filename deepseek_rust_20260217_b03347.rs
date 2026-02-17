// Represents a point in your 5D bioscale coordinate system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BiophysicalState {
    pub energy: f64,          // E
    pub protein_marker: f64,  // M_prot (e.g., from wearable biosensor)
    pub stress_index: f64,    // S_bio (composite from HRV, etc.)
    pub theta: f64,           // θ (e.g., brainwave ratio or phase)
    pub delta_temp: f64,      // ΔT (skin temperature deviation)
}

// Defines a safe corridor for each coordinate, derived from your DEFAULTBIOPHYSEVIDENCE
#[derive(Debug, Clone)]
pub struct BiophysicalCorridor {
    pub energy: (f64, f64),
    pub protein_marker: (f64, f64),
    pub stress_index: (f64, f64),
    pub theta: (f64, f64),
    pub delta_temp: (f64, f64),
}

impl BiophysicalCorridor {
    pub fn contains(&self, state: &BiophysicalState) -> bool {
        state.energy >= self.energy.0 && state.energy <= self.energy.1 &&
        state.protein_marker >= self.protein_marker.0 && state.protein_marker <= self.protein_marker.1 &&
        // ... etc.
    }
}