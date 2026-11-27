//! Step 2: feasible rates, limiting constraints, outage undelivered volume.

use network_hydraulics::{
    evaluate_two_pad_capacity, two_pad_gathering, ConstraintStatus, TwoPadScenario,
};

#[test]
fn capacity_report_names_limiting_constraint_when_both_breach() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let both = report.case(TwoPadScenario::BothPads).unwrap();
    assert!(
        !both.result.feasible(),
        "nominated both-pads plan must breach"
    );
    assert!(matches!(both.result.status, ConstraintStatus::Breached));
    assert!(both.result.limiting_node.is_some());
    assert!(
        both.max_equal_pad_rate_kg_s < case.pad_rate_kg_s,
        "feasible equal rate must sit below the nominated breach rate"
    );
}

#[test]
fn upgrade_raises_feasible_equal_rate_above_nomination() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let upgrade = report.case(TwoPadScenario::BothPadsWithUpgrade).unwrap();
    assert!(upgrade.result.feasible());
    assert!(upgrade.max_equal_pad_rate_kg_s >= case.pad_rate_kg_s);
}

#[test]
fn outage_reduces_deliverability_and_reports_undelivered_mass() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let base = report.case(TwoPadScenario::BothPads).unwrap();
    let outage = report.case(TwoPadScenario::BothPadsCompressorOutage).unwrap();

    assert!(
        outage.deliverable_total_kg_s < base.max_equal_pad_rate_kg_s * 2.0
            || !outage.result.feasible(),
        "outage must not improve deliverability vs base"
    );
    assert!(
        outage.max_equal_pad_rate_kg_s <= base.max_equal_pad_rate_kg_s + 1e-9,
        "outage feasible rate must not exceed base"
    );
    assert!(
        report.outage.undelivered_kg > 0.0,
        "48 h outage at nomination should leave undelivered mass"
    );
    assert!(report.outage.undelivered_sm3 > 0.0);
}

#[test]
fn each_pad_alone_supports_at_least_the_nomination() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    for scenario in [TwoPadScenario::PadA, TwoPadScenario::PadB] {
        let c = report.case(scenario).unwrap();
        assert!(c.result.feasible());
        assert!(c.max_equal_pad_rate_kg_s >= case.pad_rate_kg_s);
    }
}
