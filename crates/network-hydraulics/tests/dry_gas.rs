//! Acceptance: single-pipe reference, mass conservation, two-pad breach pattern.

use network_hydraulics::{
    isothermal_upstream_pressure, mass_flow_from_pressures, two_pad_gathering, ConstraintStatus,
    GasProperties, PipeGeometry, TwoPadScenario,
};
use pipeline_physics::{flow_velocity, friction_factor, reynolds_number};

fn assert_close(a: f64, b: f64, rel: f64) {
    let scale = a.abs().max(b.abs()).max(1.0);
    assert!(
        (a - b).abs() <= rel * scale,
        "expected {b}, got {a} (rel tol {rel})"
    );
}

/// Independent single-pipe reference: iterate f once from a guessed mean density,
/// then close with the same isothermal identity the solver uses.
#[test]
fn single_pipe_matches_independent_reference() {
    let gas = GasProperties::methane_standard();
    let pipe = PipeGeometry::new(5_000.0, 0.20);
    let p_down = 4.0e6;
    let m = 2.5_f64;

    let solved = isothermal_upstream_pressure(p_down, m, &pipe, &gas).unwrap();

    // Reference: recover mass flow from (solved, p_down) and compare to m.
    let m_back = mass_flow_from_pressures(solved, p_down, &pipe, &gas).unwrap();
    assert_close(m_back, m, 1e-6);

    // Second reference: rebuild P_up from the isothermal identity with f taken
    // at the converged mean pressure (does not call isothermal_upstream_pressure).
    let p_avg = 0.5 * (solved + p_down);
    let rho = gas.density_at(p_avg);
    let nu = gas.dynamic_viscosity_pa_s / rho;
    let area = std::f64::consts::PI * pipe.diameter_m * pipe.diameter_m / 4.0;
    let g = m / area;
    let v = flow_velocity(m / rho, pipe.diameter_m);
    let re = reynolds_number(v, pipe.diameter_m, nu);
    let f = friction_factor(re, pipe.roughness_m, pipe.diameter_m);
    let drop = f
        * (pipe.length_m / pipe.diameter_m)
        * gas.z_factor
        * gas.specific_gas_constant
        * gas.temperature_k
        * g
        * g;
    let p_up_ref = (p_down * p_down + drop).sqrt();
    assert_close(solved, p_up_ref, 1e-5);
}

#[test]
fn two_pad_mass_conserved_at_junctions() {
    let case = two_pad_gathering();
    let result = case.solve(TwoPadScenario::BothPads).unwrap();

    let lat_a = result.edge_mass_flow("lat_a").unwrap();
    let lat_b = result.edge_mass_flow("lat_b").unwrap();
    let trunk = result.edge_mass_flow("trunk").unwrap();
    let comp = result.edge_mass_flow("comp").unwrap();
    let sales = result.edge_mass_flow("sales_pipe").unwrap();

    assert_close(lat_a, case.pad_rate_kg_s, 1e-12);
    assert_close(lat_b, case.pad_rate_kg_s, 1e-12);
    assert_close(trunk, lat_a + lat_b, 1e-12);
    assert_close(comp, trunk, 1e-12);
    assert_close(sales, comp, 1e-12);
}

#[test]
fn each_pad_alone_is_feasible() {
    let case = two_pad_gathering();
    let a = case.solve(TwoPadScenario::PadA).unwrap();
    let b = case.solve(TwoPadScenario::PadB).unwrap();
    assert!(a.feasible(), "pad A margins: {:?}", a.constraint_margin_pa);
    assert!(b.feasible(), "pad B margins: {:?}", b.constraint_margin_pa);
    assert!(matches!(a.status, ConstraintStatus::Feasible));
    assert!(matches!(b.status, ConstraintStatus::Feasible));
}

#[test]
fn both_pads_breach_pressure_limit() {
    let case = two_pad_gathering();
    let both = case.solve(TwoPadScenario::BothPads).unwrap();
    assert!(
        !both.feasible(),
        "expected breach, margins {:?}",
        both.constraint_margin_pa
    );
    assert!(matches!(both.status, ConstraintStatus::Breached));
    assert!(both.limiting_node.is_some());
}

#[test]
fn compressor_upgrade_clears_both_pads() {
    let case = two_pad_gathering();
    let upgraded = case.solve(TwoPadScenario::BothPadsWithUpgrade).unwrap();
    assert!(
        upgraded.feasible(),
        "upgrade should clear breach: {:?}",
        upgraded.constraint_margin_pa
    );
}
