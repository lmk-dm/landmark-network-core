//! Hydraulic attributes layered onto a `network-core` topology.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::gas::{GasProperties, PipeGeometry};

/// Fixed pressure boost across a compressor edge (Pa).
///
/// Flow direction is suction (`from`) → discharge (`to`). Walking upstream from
/// sales: `P_suction = P_discharge − boost_pa`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressorSpec {
    pub boost_pa: f64,
}

/// Maximum allowable absolute pressure at a node (Pa).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PressureLimit {
    pub max_pa: f64,
}

/// Geometry and equipment keyed by `network-core` edge / node ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HydraulicSpec {
    pub gas: GasProperties,
    /// edge_id → pipe geometry (required for every `EdgeKind::Pipe`)
    pub pipes: BTreeMap<String, PipeGeometry>,
    /// edge_id → compressor (required for every `EdgeKind::Compressor`)
    pub compressors: BTreeMap<String, CompressorSpec>,
    /// node_id → pressure ceiling
    pub pressure_limits: BTreeMap<String, PressureLimit>,
}

impl HydraulicSpec {
    pub fn new(gas: GasProperties) -> Self {
        Self {
            gas,
            pipes: BTreeMap::new(),
            compressors: BTreeMap::new(),
            pressure_limits: BTreeMap::new(),
        }
    }

    pub fn with_pipe(mut self, edge_id: impl Into<String>, pipe: PipeGeometry) -> Self {
        self.pipes.insert(edge_id.into(), pipe);
        self
    }

    pub fn with_compressor(mut self, edge_id: impl Into<String>, boost_pa: f64) -> Self {
        self.compressors
            .insert(edge_id.into(), CompressorSpec { boost_pa });
        self
    }

    pub fn with_pressure_limit(mut self, node_id: impl Into<String>, max_pa: f64) -> Self {
        self.pressure_limits
            .insert(node_id.into(), PressureLimit { max_pa });
        self
    }
}
