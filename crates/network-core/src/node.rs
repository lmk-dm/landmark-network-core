//! Physical assets that occupy network junctions.

use serde::{Deserialize, Serialize};

use crate::fluid::FluidService;

/// Stable identifier for a node within one network model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// What kind of physical asset the node represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Pipe junction / manifold with no equipment meaning beyond connectivity.
    Junction,
    /// Producing or injecting well tie-in.
    Well,
    /// Separation, compression, treating, or battery facility.
    Facility,
    /// Boundary where fluid leaves the modelled estate (sales, disposal, export).
    Sink,
    /// Boundary where fluid enters from outside the modelled estate.
    Source,
    /// Explicit valve station when isolation identity matters.
    ValveStation,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Junction => "junction",
            Self::Well => "well",
            Self::Facility => "facility",
            Self::Sink => "sink",
            Self::Source => "source",
            Self::ValveStation => "valve_station",
        }
    }
}

/// A node in the field network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Dominant fluid at this asset when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fluid: Option<FluidService>,
    /// Optional human label; identity hashing ignores this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Node {
    pub fn new(id: impl Into<NodeId>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            fluid: None,
            name: None,
        }
    }

    pub fn with_fluid(mut self, fluid: FluidService) -> Self {
        self.fluid = Some(fluid);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
