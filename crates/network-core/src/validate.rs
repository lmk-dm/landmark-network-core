//! Structural validation of a network model.

use std::collections::HashSet;

use crate::error::{NetworkError, Result};
use crate::topology::NetworkModel;

/// Refuse models that cannot support a truthful analysis.
pub fn validate_topology(model: &NetworkModel) -> Result<()> {
    if model.nodes.is_empty() {
        return Err(NetworkError::EmptyNetwork);
    }

    let mut node_ids = HashSet::with_capacity(model.nodes.len());
    for node in &model.nodes {
        if node.id.as_str().is_empty() {
            return Err(NetworkError::EmptyId { kind: "node" });
        }
        if !node_ids.insert(node.id.as_str()) {
            return Err(NetworkError::DuplicateNode(node.id.0.clone()));
        }
    }

    let mut edge_ids = HashSet::with_capacity(model.edges.len());
    for edge in &model.edges {
        if edge.id.as_str().is_empty() {
            return Err(NetworkError::EmptyId { kind: "edge" });
        }
        if !edge_ids.insert(edge.id.as_str()) {
            return Err(NetworkError::DuplicateEdge(edge.id.0.clone()));
        }
        if edge.from == edge.to {
            return Err(NetworkError::SelfLoop(edge.id.0.clone()));
        }
        if !node_ids.contains(edge.from.as_str()) {
            return Err(NetworkError::MissingNode {
                edge: edge.id.0.clone(),
                node: edge.from.0.clone(),
            });
        }
        if !node_ids.contains(edge.to.as_str()) {
            return Err(NetworkError::MissingNode {
                edge: edge.id.0.clone(),
                node: edge.to.0.clone(),
            });
        }
    }

    Ok(())
}
