//! Persistent field network: the unit of analysis for Network Intelligence.

use serde::{Deserialize, Serialize};

use crate::edge::{Edge, EdgeId};
use crate::error::{NetworkError, Result};
use crate::identity::content_sha256;
use crate::node::{Node, NodeId};
use crate::validate::validate_topology;

/// One field (or regional) network model.
///
/// The network exists independently of any particular study. Hydraulics,
/// reliability, and economics engines take a validated `NetworkModel` as input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkModel {
    /// Stable product identity for this network (e.g. field or gathering system).
    pub network_id: String,
    /// ISO-8601 instant or date the topology is as-of. Absence is allowed only
    /// for synthetic fixtures; production builders should set it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl NetworkModel {
    pub fn new(network_id: impl Into<String>) -> Self {
        Self {
            network_id: network_id.into(),
            as_of: None,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_as_of(mut self, as_of: impl Into<String>) -> Self {
        self.as_of = Some(as_of.into());
        self
    }

    pub fn add_node(&mut self, node: Node) -> Result<()> {
        if node.id.as_str().is_empty() {
            return Err(NetworkError::EmptyId { kind: "node" });
        }
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(NetworkError::DuplicateNode(node.id.0));
        }
        self.nodes.push(node);
        Ok(())
    }

    pub fn add_edge(&mut self, edge: Edge) -> Result<()> {
        if edge.id.as_str().is_empty() {
            return Err(NetworkError::EmptyId { kind: "edge" });
        }
        if edge.from == edge.to {
            return Err(NetworkError::SelfLoop(edge.id.0));
        }
        if self.edges.iter().any(|e| e.id == edge.id) {
            return Err(NetworkError::DuplicateEdge(edge.id.0));
        }
        self.edges.push(edge);
        Ok(())
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == *id)
    }

    pub fn edge(&self, id: &EdgeId) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == *id)
    }

    /// Structural check before any physics or economics engine runs.
    pub fn validate(&self) -> Result<()> {
        validate_topology(self)
    }

    /// Content hash of topology meaning (ids, kinds, connectivity, fluid).
    ///
    /// Display names are excluded so renaming a facility does not change the
    /// decision identity of an otherwise identical network.
    pub fn topology_sha256(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(1 + self.nodes.len() + self.edges.len());
        lines.push(format!(
            "network|{}|{}",
            self.network_id,
            self.as_of.as_deref().unwrap_or("-")
        ));

        let mut nodes: Vec<&Node> = self.nodes.iter().collect();
        nodes.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        for n in nodes {
            lines.push(format!(
                "node|{}|{}|{}",
                n.id.as_str(),
                n.kind.as_str(),
                n.fluid.map(|f| f.as_str()).unwrap_or("-")
            ));
        }

        let mut edges: Vec<&Edge> = self.edges.iter().collect();
        edges.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        for e in edges {
            lines.push(format!(
                "edge|{}|{}|{}|{}|{}",
                e.id.as_str(),
                e.kind.as_str(),
                e.from.as_str(),
                e.to.as_str(),
                e.fluid.as_str()
            ));
        }

        content_sha256(lines.join("\n").as_bytes())
    }
}
