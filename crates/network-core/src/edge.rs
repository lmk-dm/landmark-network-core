//! Directed connections between network nodes.

use serde::{Deserialize, Serialize};

use crate::fluid::FluidService;
use crate::node::NodeId;

/// Stable identifier for an edge within one network model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

impl EdgeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EdgeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for EdgeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// What kind of physical connection the edge represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Pipe,
    Pump,
    Compressor,
    Valve,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pipe => "pipe",
            Self::Pump => "pump",
            Self::Compressor => "compressor",
            Self::Valve => "valve",
        }
    }
}

/// A directed edge in the field network.
///
/// Direction is the design flow sense. Contingency and reverse-flow studies may
/// reinterpret it; they do not mutate this record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub kind: EdgeKind,
    pub from: NodeId,
    pub to: NodeId,
    pub fluid: FluidService,
    /// Optional human label; identity hashing ignores this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Edge {
    pub fn new(
        id: impl Into<EdgeId>,
        kind: EdgeKind,
        from: impl Into<NodeId>,
        to: impl Into<NodeId>,
        fluid: FluidService,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            from: from.into(),
            to: to.into(),
            fluid,
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
