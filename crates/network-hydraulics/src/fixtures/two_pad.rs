//! Two-pad gas gathering fixture for the first tie-in decision.
//!
//! ```text
//! pad_a --lat_a--+
//!                ctb --trunk--> suction --comp--> discharge --sales_pipe--> sales
//! pad_b --lat_b--+
//! ```
//!
//! Each pad alone is feasible against the pressure ceilings. Both pads at the
//! nominated rate together breach the pad / CTB pressure limit. A compressor
//! upgrade (higher boost) restores feasibility for both pads.

use std::collections::BTreeMap;

use network_core::{Edge, EdgeKind, FluidService, NetworkModel, Node, NodeId, NodeKind};

use crate::error::Result;
use crate::gas::{GasProperties, PipeGeometry};
use crate::result::SteadyStateResult;
use crate::solve::{solve_dry_gas_tree, DryGasProblem};
use crate::spec::HydraulicSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoPadScenario {
    /// Pad A only at the nominated rate.
    PadA,
    /// Pad B only at the nominated rate.
    PadB,
    /// Both pads at the nominated rate (breaches without upgrade).
    BothPads,
    /// Both pads with the compressor expansion boost.
    BothPadsWithUpgrade,
    /// Both pads with the compressor offline (boost = 0).
    BothPadsCompressorOutage,
}

#[derive(Debug, Clone)]
pub struct TwoPadCase {
    pub topology: NetworkModel,
    pub hydraulics: HydraulicSpec,
    pub sink_id: String,
    pub sink_pressure_pa: f64,
    /// Nominated injection per pad, kg/s.
    pub pad_rate_kg_s: f64,
    pub base_boost_pa: f64,
    pub upgrade_boost_pa: f64,
}

impl TwoPadCase {
    pub fn injections(&self, scenario: TwoPadScenario) -> BTreeMap<String, f64> {
        let mut m = BTreeMap::new();
        match scenario {
            TwoPadScenario::PadA => {
                m.insert("pad_a".into(), self.pad_rate_kg_s);
            }
            TwoPadScenario::PadB => {
                m.insert("pad_b".into(), self.pad_rate_kg_s);
            }
            TwoPadScenario::BothPads
            | TwoPadScenario::BothPadsWithUpgrade
            | TwoPadScenario::BothPadsCompressorOutage => {
                m.insert("pad_a".into(), self.pad_rate_kg_s);
                m.insert("pad_b".into(), self.pad_rate_kg_s);
            }
        }
        m
    }

    pub fn hydraulics_for(&self, scenario: TwoPadScenario) -> HydraulicSpec {
        let mut h = self.hydraulics.clone();
        match scenario {
            TwoPadScenario::BothPadsWithUpgrade => {
                h.compressors.get_mut("comp").expect("comp").boost_pa = self.upgrade_boost_pa;
            }
            TwoPadScenario::BothPadsCompressorOutage => {
                h.compressors.get_mut("comp").expect("comp").boost_pa = 0.0;
            }
            _ => {}
        }
        h
    }

    pub fn solve(&self, scenario: TwoPadScenario) -> Result<SteadyStateResult> {
        let hydraulics = self.hydraulics_for(scenario);
        let injections = self.injections(scenario);
        solve_dry_gas_tree(&DryGasProblem {
            topology: &self.topology,
            hydraulics: &hydraulics,
            sink_id: NodeId(self.sink_id.clone()),
            sink_pressure_pa: self.sink_pressure_pa,
            injections_kg_s: injections,
        })
    }
}

/// Build the canonical two-pad gathering case.
///
/// Pressure ceilings are placed between the solved single-pad and dual-pad
/// peaks (base compressor), so:
/// - pad A alone / pad B alone → feasible
/// - both pads, base compressor → CTB/suction breach
/// - both pads, upgraded compressor → feasible again
pub fn two_pad_gathering() -> TwoPadCase {
    let mut topology = NetworkModel::new("two-pad-gas-gathering").with_as_of("2026-11-26");
    topology
        .add_node(Node::new("pad_a", NodeKind::Well).with_fluid(FluidService::Gas))
        .unwrap();
    topology
        .add_node(Node::new("pad_b", NodeKind::Well).with_fluid(FluidService::Gas))
        .unwrap();
    topology
        .add_node(Node::new("ctb", NodeKind::Facility).with_fluid(FluidService::Gas))
        .unwrap();
    topology
        .add_node(Node::new("suction", NodeKind::Junction).with_fluid(FluidService::Gas))
        .unwrap();
    topology
        .add_node(Node::new("discharge", NodeKind::Junction).with_fluid(FluidService::Gas))
        .unwrap();
    topology
        .add_node(Node::new("sales", NodeKind::Sink).with_fluid(FluidService::Gas))
        .unwrap();

    topology
        .add_edge(Edge::new(
            "lat_a",
            EdgeKind::Pipe,
            "pad_a",
            "ctb",
            FluidService::Gas,
        ))
        .unwrap();
    topology
        .add_edge(Edge::new(
            "lat_b",
            EdgeKind::Pipe,
            "pad_b",
            "ctb",
            FluidService::Gas,
        ))
        .unwrap();
    topology
        .add_edge(Edge::new(
            "trunk",
            EdgeKind::Pipe,
            "ctb",
            "suction",
            FluidService::Gas,
        ))
        .unwrap();
    topology
        .add_edge(Edge::new(
            "comp",
            EdgeKind::Compressor,
            "suction",
            "discharge",
            FluidService::Gas,
        ))
        .unwrap();
    topology
        .add_edge(Edge::new(
            "sales_pipe",
            EdgeKind::Pipe,
            "discharge",
            "sales",
            FluidService::Gas,
        ))
        .unwrap();

    let base_boost_pa = 0.4e6;
    let upgrade_boost_pa = 2.6e6;
    let pad_rate_kg_s = 3.0;
    let sink_pressure_pa = 3.0e6;

    // Generous limits while we measure single vs dual peaks.
    let mut hydraulics = HydraulicSpec::new(GasProperties::methane_standard())
        .with_pipe("lat_a", PipeGeometry::new(2_000.0, 0.2540))
        .with_pipe("lat_b", PipeGeometry::new(2_000.0, 0.2540))
        .with_pipe("trunk", PipeGeometry::new(10_000.0, 0.2032))
        .with_pipe("sales_pipe", PipeGeometry::new(1_500.0, 0.3048))
        .with_compressor("comp", base_boost_pa)
        .with_pressure_limit("pad_a", 50.0e6)
        .with_pressure_limit("pad_b", 50.0e6)
        .with_pressure_limit("ctb", 50.0e6)
        .with_pressure_limit("suction", 50.0e6);

    let mut case = TwoPadCase {
        topology,
        hydraulics: hydraulics.clone(),
        sink_id: "sales".into(),
        sink_pressure_pa,
        pad_rate_kg_s,
        base_boost_pa,
        upgrade_boost_pa,
    };

    let alone = case
        .solve(TwoPadScenario::PadA)
        .expect("single-pad solve for limit placement");
    let both = case
        .solve(TwoPadScenario::BothPads)
        .expect("dual-pad solve for limit placement");
    let upgraded = case
        .solve(TwoPadScenario::BothPadsWithUpgrade)
        .expect("upgrade solve for limit placement");

    let p_ctb_alone = alone.node_pressure("ctb").unwrap();
    let p_ctb_both = both.node_pressure("ctb").unwrap();
    let p_ctb_up = upgraded.node_pressure("ctb").unwrap();
    let p_suc_alone = alone.node_pressure("suction").unwrap();
    let p_suc_both = both.node_pressure("suction").unwrap();
    let p_suc_up = upgraded.node_pressure("suction").unwrap();

    let p_max_ctb = ceiling_between(&[p_ctb_alone, p_ctb_up], p_ctb_both, "ctb");
    let p_max_suction = ceiling_between(&[p_suc_alone, p_suc_up], p_suc_both, "suction");
    let p_max_pad = 50.0e6;

    hydraulics.pressure_limits.clear();
    hydraulics = hydraulics
        .with_pressure_limit("pad_a", p_max_pad)
        .with_pressure_limit("pad_b", p_max_pad)
        .with_pressure_limit("ctb", p_max_ctb)
        .with_pressure_limit("suction", p_max_suction);
    case.hydraulics = hydraulics;
    case
}

/// Place a limit above every feasible peak and strictly below the breach peak.
fn ceiling_between(feasible_peaks: &[f64], breach_peak: f64, label: &str) -> f64 {
    let lo = feasible_peaks
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        lo < breach_peak,
        "fixture {label}: feasible peak {lo} must sit below breach peak {breach_peak}"
    );
    0.5 * (lo + breach_peak)
}
