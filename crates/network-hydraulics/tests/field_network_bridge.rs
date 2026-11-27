//! Field Network must consume [`dump_two_pad_capacity`], not hand-copied Mcf figures.

use network_hydraulics::{
    dump_two_pad_capacity, evaluate_two_pad_capacity, two_pad_gathering, TwoPadScenario,
    TWO_PAD_CAPACITY_SCHEMA,
};

#[test]
fn dump_contract_matches_evaluated_rates() {
    let case = two_pad_gathering();
    let gas = &case.hydraulics.gas;
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let dump = dump_two_pad_capacity(48.0).unwrap();

    assert_eq!(dump.schema, TWO_PAD_CAPACITY_SCHEMA);
    assert_eq!(dump.fixture, "two_pad_gathering");

    let nominated = gas.kg_s_to_mcf_d(case.pad_rate_kg_s);
    let base = report.case(TwoPadScenario::BothPads).unwrap();
    let upgrade = report.case(TwoPadScenario::BothPadsWithUpgrade).unwrap();

    assert!((dump.nominated_mcf_d_per_pad - nominated).abs() < 1e-9);
    assert!(
        (dump.base_max_equal_mcf_d - gas.kg_s_to_mcf_d(base.max_equal_pad_rate_kg_s)).abs() < 1e-9
    );
    assert!(
        (dump.upgrade_max_equal_mcf_d - gas.kg_s_to_mcf_d(upgrade.max_equal_pad_rate_kg_s)).abs()
            < 1e-9
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
