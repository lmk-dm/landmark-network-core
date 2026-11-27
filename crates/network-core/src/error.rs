//! Refusals for an incomplete or inconsistent network model.

use thiserror::Error;

/// Failures that mean the network cannot be trusted as a decision input.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NetworkError {
    /// An empty graph is not a network — every analysis would be vacuously true.
    #[error("network has no nodes; a field model requires at least one asset")]
    EmptyNetwork,

    /// Two assets claim the same identity.
    #[error("duplicate node id {0}")]
    DuplicateNode(String),

    /// Two connections claim the same identity.
    #[error("duplicate edge id {0}")]
    DuplicateEdge(String),

    /// An edge references a node that is not in the model.
    #[error("edge {edge} references missing node {node}")]
    MissingNode { edge: String, node: String },

    /// A node or edge id was blank.
    #[error("{kind} id must not be empty")]
    EmptyId { kind: &'static str },

    /// from and to must differ; a self-loop is not a field connection.
    #[error("edge {0} connects a node to itself")]
    SelfLoop(String),
}

pub type Result<T> = std::result::Result<T, NetworkError>;
