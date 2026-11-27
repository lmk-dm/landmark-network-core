//! Steady-state solve outputs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeFlow {
    pub edge_id: String,
    pub mass_flow_kg_s: f64,
    /// Pressure at the upstream (`from`) end after the solve, Pa.
    pub p_from_pa: f64,
    /// Pressure at the downstream (`to`) end after the solve, Pa.
    pub p_to_pa: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintStatus {
    Feasible,
    Breached,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteadyStateResult {
    pub status: ConstraintStatus,
    pub node_pressure_pa: BTreeMap<String, f64>,
    pub edge_flows: BTreeMap<String, EdgeFlow>,
    /// `limit − pressure` at each limited node. Negative ⇒ breach.
    pub constraint_margin_pa: BTreeMap<String, f64>,
    /// Node id of the most violated (most negative) margin, if any breach.
    pub limiting_node: Option<String>,
}

impl SteadyStateResult {
    pub fn feasible(&self) -> bool {
        matches!(self.status, ConstraintStatus::Feasible)
    }

    pub fn edge_mass_flow(&self, edge_id: &str) -> Option<f64> {
        self.edge_flows.get(edge_id).map(|e| e.mass_flow_kg_s)
    }

    pub fn node_pressure(&self, node_id: &str) -> Option<f64> {
        self.node_pressure_pa.get(node_id).copied()
    }
}
