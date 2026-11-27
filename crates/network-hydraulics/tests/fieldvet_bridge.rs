//! Keep FieldVET's embedded Mcf capacities aligned with this fixture.

use network_hydraulics::{evaluate_two_pad_capacity, two_pad_gathering, TwoPadScenario};

/// Values copied into `landmark_fieldvet.examples.two_pad_tie_in`.
const FIELDVET_NOMINATED_MCF_D: f64 = 13_491.906708;
const FIELDVET_BASE_MAX_EQUAL_MCF_D: f64 = 10_535.587006;
const FIELDVET_UPGRADE_MAX_EQUAL_MCF_D: f64 = 21_170.597066;

#[test]
fn fieldvet_embedded_rates_match_hydraulics() {
    let case = two_pad_gathering();
    let gas = &case.hydraulics.gas;
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();

    let nominated = gas.kg_s_to_mcf_d(case.pad_rate_kg_s);
    let base = report.case(TwoPadScenario::BothPads).unwrap();
    let upgrade = report.case(TwoPadScenario::BothPadsWithUpgrade).unwrap();

    assert!((nominated - FIELDVET_NOMINATED_MCF_D).abs() < 1e-3);
    assert!((gas.kg_s_to_mcf_d(base.max_equal_pad_rate_kg_s) - FIELDVET_BASE_MAX_EQUAL_MCF_D).abs() < 1e-3);
    assert!(
        (gas.kg_s_to_mcf_d(upgrade.max_equal_pad_rate_kg_s) - FIELDVET_UPGRADE_MAX_EQUAL_MCF_D).abs()
            < 1e-3
    );
}

#[test]
fn curtailed_equal_rate_is_hydraulically_feasible() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let max_eq = report
        .case(TwoPadScenario::BothPads)
        .unwrap()
        .max_equal_pad_rate_kg_s;

    let hydraulics = case.hydraulics_for(TwoPadScenario::BothPads);
    let mut injections = case.injections(TwoPadScenario::BothPads);
    for v in injections.values_mut() {
        *v = max_eq;
    }
    let result = network_hydraulics::solve_dry_gas_tree(&network_hydraulics::DryGasProblem {
        topology: &case.topology,
        hydraulics: &hydraulics,
        sink_id: network_core::NodeId(case.sink_id.clone()),
        sink_pressure_pa: case.sink_pressure_pa,
        injections_kg_s: injections,
    })
    .unwrap();
    assert!(result.feasible());
}
