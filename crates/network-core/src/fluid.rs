//! Fluid service carried by a network segment.
//!
//! This labels *what the network moves*. It is not a PVT model.

use serde::{Deserialize, Serialize};

/// Dominant service on a node or edge.
///
/// Multiphase is an explicit service, not a silent default: analyses that only
/// handle single-phase oil/gas/water must refuse multiphase edges rather than
/// invent an equivalent fluid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FluidService {
    Oil,
    Gas,
    Water,
    Multiphase,
    Co2,
    Hydrogen,
}

impl FluidService {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Oil => "oil",
            Self::Gas => "gas",
            Self::Water => "water",
            Self::Multiphase => "multiphase",
            Self::Co2 => "co2",
            Self::Hydrogen => "hydrogen",
        }
    }
}
