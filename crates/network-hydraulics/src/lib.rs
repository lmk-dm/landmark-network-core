//! Steady-state dry-gas hydraulics for Landmark Network Intelligence.
//!
//! Segment friction comes from `pipeline-physics` (Reynolds + Darcy / Swamee-Jain).
//! Compressible isothermal pipe integration and branched-tree network assembly
//! live here. Multiphase and looped networks are out of scope for this crate
//! until a later engine owns them.
//!
//! # Fixed-plan solve
//!
//! Given pad injection rates and a fixed sales pressure, assign unique pipe
//! rates on a gathering tree, walk pressures upstream, and report constraint
//! margins against node pressure ceilings.

#![forbid(unsafe_code)]

pub mod capacity;
pub mod error;
pub mod fixtures;
pub mod gas;
pub mod result;
pub mod solve;
pub mod spec;

pub use capacity::{
    evaluate_two_pad_capacity, max_equal_pad_rate, CapacityCase, CapacityReport, OutageImpact,
};
pub use error::{HydraulicsError, Result};
pub use fixtures::two_pad::{two_pad_gathering, TwoPadCase, TwoPadScenario};
pub use gas::{
    isothermal_upstream_pressure, mass_flow_from_pressures, pipe_area, GasProperties, PipeGeometry,
};
pub use result::{ConstraintStatus, EdgeFlow, SteadyStateResult};
pub use solve::{solve_dry_gas_tree, DryGasProblem};
pub use spec::{CompressorSpec, HydraulicSpec, PressureLimit};
