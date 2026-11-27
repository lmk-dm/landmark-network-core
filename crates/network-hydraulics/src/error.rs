//! Refusals for hydraulics that cannot be trusted as a decision input.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum HydraulicsError {
    #[error(transparent)]
    Topology(#[from] network_core::NetworkError),

    #[error("network is not a gathering tree rooted at sink {sink}: {reason}")]
    NotATree { sink: String, reason: String },

    #[error("missing pipe geometry for edge {0}")]
    MissingPipe(String),

    #[error("missing compressor specification for edge {0}")]
    MissingCompressor(String),

    #[error("sink node {0} is not in the topology")]
    UnknownSink(String),

    #[error("injection specified for unknown node {0}")]
    UnknownInjection(String),

    #[error("edge {edge} mass flow {flow_kg_s} kg/s is not physically solvable: {reason}")]
    UnsolvablePipe {
        edge: String,
        flow_kg_s: f64,
        reason: String,
    },

    #[error("compressor {edge} implies non-positive suction pressure ({suction_pa} Pa)")]
    CompressorSuctionNonPositive { edge: String, suction_pa: f64 },

    #[error("{0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, HydraulicsError>;
