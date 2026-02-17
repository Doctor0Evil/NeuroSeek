//! Biophysical state and safety corridors.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BiophysicalState {
    pub e: f64,
    pub m_prot: f64,
    pub s_bio: f64,
    pub theta: f64,
    pub t: f64,
}

impl BiophysicalState {
    pub fn new(e: f64, m_prot: f64, s_bio: f64, theta: f64, t: f64) -> Self {
        Self { e, m_prot, s_bio, theta, t }
    }
}

#[derive(Debug, Clone)]
pub struct BiophysicalCorridor {
    pub e: (f64, f64),
    pub m_prot: (f64, f64),
    pub s_bio: (f64, f64),
    pub theta: (f64, f64),
    pub t: (f64, f64),
}

impl BiophysicalCorridor {
    pub fn contains(&self, state: &BiophysicalState) -> bool {
        state.e >= self.e.0 && state.e <= self.e.1 &&
        state.m_prot >= self.m_prot.0 && state.m_prot <= self.m_prot.1 &&
        state.s_bio >= self.s_bio.0 && state.s_bio <= self.s_bio.1 &&
        state.theta >= self.theta.0 && state.theta <= self.theta.1 &&
        state.t >= self.t.0 && state.t <= self.t.1
    }
}
