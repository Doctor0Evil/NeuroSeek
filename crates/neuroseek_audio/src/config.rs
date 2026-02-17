//! Configuration for audio protocols: either a fixed point or a polytope region.

use crate::audio_nanopolytope::{AudioNanopolytope, AudioState};
use serde::{Deserialize, Serialize};

/// A protocol can be a single fixed dose (point) or a region (polytope) to explore.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Protocol {
    /// A single fixed audio state.
    Fixed(AudioState),
    /// A polytope of safe audio states.
    Polytope(AudioNanopolytope),
}

impl Protocol {
    /// If the protocol is a polytope, return it.
    pub fn as_polytope(&self) -> Option<&AudioNanopolytope> {
        match self {
            Protocol::Polytope(p) => Some(p),
            _ => None,
        }
    }
}
