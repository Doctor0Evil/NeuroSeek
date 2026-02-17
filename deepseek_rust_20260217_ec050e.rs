#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nanopolytope {
    pub id: String,
    pub centroid: BiophysicalState,          // The center of the stable region
    pub radius: f64,                         // How far states can deviate and still be "in" the polytope
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub occurrence_count: u32,
    pub avg_craving_reduction: Option<f64>,  // If correlated with craving biomarker
    pub tags: Vec<String>,                    // e.g., "resting", "craving_resistance", "post_exercise"
}