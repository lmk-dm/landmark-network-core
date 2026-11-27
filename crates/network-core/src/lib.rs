//! Canonical oil and gas field network model.
//!
//! `network-core` owns the persistent topology: typed nodes and edges, content
//! identity, and structural validation. Sibling engines attach physics,
//! reliability, expansion, and economics to this model. They do not redefine it.
//!
//! # Boundary
//!
//! - In: graph topology, asset identity, fluid service, structural refusals
//! - Out: hydraulics, Monte Carlo reliability, MIP expansion, HTTP, storage

#![forbid(unsafe_code)]

pub mod edge;
pub mod error;
pub mod fluid;
pub mod identity;
pub mod node;
pub mod topology;
pub mod validate;

pub use edge::{Edge, EdgeId, EdgeKind};
pub use error::{NetworkError, Result};
pub use fluid::FluidService;
pub use identity::content_sha256;
pub use node::{Node, NodeId, NodeKind};
pub use topology::NetworkModel;
pub use validate::validate_topology;
