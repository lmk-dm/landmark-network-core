use network_hydraulics::{evaluate_two_pad_capacity, two_pad_gathering};

fn main() {
    let case = two_pad_gathering();
    let report = evaluate_two_pad_capacity(&case, 48.0).unwrap();
    let gas = &case.hydraulics.gas;
    println!("pad_rate_kg_s={}", case.pad_rate_kg_s);
    println!(
        "nominated_mcf_d_per_pad={:.6}",
        gas.kg_s_to_mcf_d(case.pad_rate_kg_s)
    );
    for c in &report.cases {
        println!(
            "{} feasible={} limiting={:?} max_equal_kg_s={:.6} max_equal_mcf_d={:.6} deliverable_total_kg_s={:.6}",
            c.label,
            c.result.feasible(),
            c.result.limiting_node,
            c.max_equal_pad_rate_kg_s,
            gas.kg_s_to_mcf_d(c.max_equal_pad_rate_kg_s),
            c.deliverable_total_kg_s
        );
    }
    println!(
        "outage undelivered_kg={:.3} undelivered_sm3={:.3} undelivered_mcf={:.3}",
        report.outage.undelivered_kg,
        report.outage.undelivered_sm3,
        report.outage.undelivered_sm3 / 28.316846592
    );
}
