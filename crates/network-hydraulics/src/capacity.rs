//! Feasible deliverability from a fixed network under pad nominations.

use serde::{Deserialize, Serialize};

use crate::error::{HydraulicsError, Result};
use crate::fixtures::two_pad::{two_pad_gathering, TwoPadCase, TwoPadScenario};
use crate::result::{ConstraintStatus, SteadyStateResult};

/// Machine contract for Field Network (and other consumers) of two-pad capacities.
///
/// Produced by [`dump_two_pad_capacity`] / `cargo run -p network-hydraulics --example dump_capacity`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwoPadCapacityDump {
    pub schema: String,
    pub fixture: String,
    pub source: String,
    pub outage_duration_hours: f64,
    pub pad_rate_kg_s: f64,
    pub nominated_mcf_d_per_pad: f64,
    pub base_max_equal_mcf_d: f64,
    pub upgrade_max_equal_mcf_d: f64,
    pub outage_undelivered_mcf: f64,
}

pub const TWO_PAD_CAPACITY_SCHEMA: &str = "network-hydraulics.two_pad_capacity.v1";

/// One network configuration evaluated against the same pad nominations.
#[derive(Debug, Clone, PartialEq)]
pub struct CapacityCase {
    pub scenario: TwoPadScenario,
    pub label: &'static str,
    /// Nominated injection per active pad (kg/s).
    pub nominated_pad_rate_kg_s: f64,
    pub result: SteadyStateResult,
    /// Largest equal per-pad rate that stays feasible in this configuration.
    pub max_equal_pad_rate_kg_s: f64,
    /// Field delivery at the nominated rates when feasible; otherwise the
    /// max equal-pad deliverability (kg/s total).
    pub deliverable_total_kg_s: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutageImpact {
    pub duration_hours: f64,
    pub nominated_total_kg_s: f64,
    pub deliverable_total_kg_s: f64,
    /// Mass the field cannot deliver during the outage window (kg).
    pub undelivered_kg: f64,
    /// Standard cubic metres the field cannot deliver (Z=1, methane std).
    pub undelivered_sm3: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapacityReport {
    pub cases: Vec<CapacityCase>,
    pub outage: OutageImpact,
}

impl CapacityReport {
    pub fn case(&self, scenario: TwoPadScenario) -> Option<&CapacityCase> {
        self.cases.iter().find(|c| c.scenario == scenario)
    }
}

/// Evaluate the two-pad gathering fixture and emit the Field Network capacity contract.
pub fn dump_two_pad_capacity(outage_duration_hours: f64) -> Result<TwoPadCapacityDump> {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, outage_duration_hours)?;
    let gas = &case.hydraulics.gas;
    let base = report
        .case(TwoPadScenario::BothPads)
        .ok_or_else(|| HydraulicsError::InvalidInput("missing both_pads case".into()))?;
    let upgrade = report
        .case(TwoPadScenario::BothPadsWithUpgrade)
        .ok_or_else(|| HydraulicsError::InvalidInput("missing upgrade case".into()))?;
    // Match dump_capacity historical print: undelivered_sm3 / 28.316846592 → Mcf.
    let undelivered_mcf = report.outage.undelivered_sm3 / 28.316846592;
    Ok(TwoPadCapacityDump {
        schema: TWO_PAD_CAPACITY_SCHEMA.into(),
        fixture: "two_pad_gathering".into(),
        source: "network-hydraulics.dump_two_pad_capacity".into(),
        outage_duration_hours,
        pad_rate_kg_s: case.pad_rate_kg_s,
        nominated_mcf_d_per_pad: gas.kg_s_to_mcf_d(case.pad_rate_kg_s),
        base_max_equal_mcf_d: gas.kg_s_to_mcf_d(base.max_equal_pad_rate_kg_s),
        upgrade_max_equal_mcf_d: gas.kg_s_to_mcf_d(upgrade.max_equal_pad_rate_kg_s),
        outage_undelivered_mcf: undelivered_mcf,
    })
}

/// Evaluate base, upgrade, and outage configurations for the two-pad fixture.
pub fn evaluate_two_pad_capacity(
    case: &TwoPadCase,
    outage_duration_hours: f64,
) -> Result<CapacityReport> {
    if outage_duration_hours < 0.0 {
        return Err(HydraulicsError::InvalidInput(
            "outage duration must be non-negative".into(),
        ));
    }

    let scenarios = [
        (TwoPadScenario::PadA, "pad_a_alone"),
        (TwoPadScenario::PadB, "pad_b_alone"),
        (TwoPadScenario::BothPads, "both_pads_base"),
        (TwoPadScenario::BothPadsWithUpgrade, "both_pads_upgrade"),
        (
            TwoPadScenario::BothPadsCompressorOutage,
            "both_pads_outage",
        ),
    ];

    let mut cases = Vec::with_capacity(scenarios.len());
    for (scenario, label) in scenarios {
        let nominated = case.pad_rate_kg_s;
        let result = case.solve(scenario)?;
        let max_equal = max_equal_pad_rate(case, scenario)?;
        let active_pads = match scenario {
            TwoPadScenario::PadA | TwoPadScenario::PadB => 1.0,
            _ => 2.0,
        };
        let deliverable_total = if result.feasible() {
            nominated * active_pads
        } else {
            max_equal * active_pads
        };
        cases.push(CapacityCase {
            scenario,
            label,
            nominated_pad_rate_kg_s: nominated,
            result,
            max_equal_pad_rate_kg_s: max_equal,
            deliverable_total_kg_s: deliverable_total,
        });
    }

    let base = cases
        .iter()
        .find(|c| c.scenario == TwoPadScenario::BothPads)
        .expect("both pads case");
    let outage = cases
        .iter()
        .find(|c| c.scenario == TwoPadScenario::BothPadsCompressorOutage)
        .expect("outage case");

    let nominated_total = case.pad_rate_kg_s * 2.0;
    let deliverable = outage.deliverable_total_kg_s;
    let shortfall_rate = (nominated_total - deliverable).max(0.0);
    let undelivered_kg = shortfall_rate * outage_duration_hours * 3600.0;
    let rho_std = case.hydraulics.gas.density_at(101_325.0);
    let undelivered_sm3 = undelivered_kg / rho_std;

    // Reference: base deliverability at nomination when feasible, else its max.
    let _base_delivery = base.deliverable_total_kg_s;

    Ok(CapacityReport {
        cases,
        outage: OutageImpact {
            duration_hours: outage_duration_hours,
            nominated_total_kg_s: nominated_total,
            deliverable_total_kg_s: deliverable,
            undelivered_kg,
            undelivered_sm3,
        },
    })
}

/// Binary-search the largest equal per-pad rate that remains feasible.
pub fn max_equal_pad_rate(case: &TwoPadCase, scenario: TwoPadScenario) -> Result<f64> {
    let mut lo = 0.0;
    let mut hi = case.pad_rate_kg_s;
    // Expand upper bound until infeasible or a generous cap.
    for _ in 0..24 {
        if rate_feasible(case, scenario, hi)? {
            lo = hi;
            hi *= 2.0;
            if hi > case.pad_rate_kg_s * 64.0 {
                return Ok(hi);
            }
        } else {
            break;
        }
    }
    if rate_feasible(case, scenario, hi)? {
        return Ok(hi);
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if rate_feasible(case, scenario, mid)? {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(lo)
}

fn rate_feasible(case: &TwoPadCase, scenario: TwoPadScenario, pad_rate: f64) -> Result<bool> {
    let hydraulics = case.hydraulics_for(scenario);
    let mut injections = case.injections(scenario);
    for v in injections.values_mut() {
        *v = pad_rate;
    }
    // PadA / PadB scenarios only set one pad; keep that.
    if matches!(scenario, TwoPadScenario::PadA) {
        injections.insert("pad_a".into(), pad_rate);
        injections.remove("pad_b");
    } else if matches!(scenario, TwoPadScenario::PadB) {
        injections.insert("pad_b".into(), pad_rate);
        injections.remove("pad_a");
    }
    let result = crate::solve::solve_dry_gas_tree(&crate::solve::DryGasProblem {
        topology: &case.topology,
        hydraulics: &hydraulics,
        sink_id: network_core::NodeId(case.sink_id.clone()),
        sink_pressure_pa: case.sink_pressure_pa,
        injections_kg_s: injections,
    })?;
    Ok(matches!(result.status, ConstraintStatus::Feasible))
}
