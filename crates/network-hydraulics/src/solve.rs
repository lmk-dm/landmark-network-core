//! Branched dry-gas gathering solve for a fixed injection plan.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use network_core::{EdgeKind, NetworkModel, NodeId};

use crate::error::{HydraulicsError, Result};
use crate::gas::isothermal_upstream_pressure;
use crate::result::{ConstraintStatus, EdgeFlow, SteadyStateResult};
use crate::spec::HydraulicSpec;

/// Fixed-plan dry-gas problem on a gathering tree.
#[derive(Debug, Clone)]
pub struct DryGasProblem<'a> {
    pub topology: &'a NetworkModel,
    pub hydraulics: &'a HydraulicSpec,
    pub sink_id: NodeId,
    pub sink_pressure_pa: f64,
    /// node_id → injection mass flow (kg/s). Non-pad nodes should be absent or 0.
    pub injections_kg_s: BTreeMap<String, f64>,
}

/// Solve pressures and rates for a fixed injection plan.
///
/// Requires a directed gathering tree: every edge's `to` is strictly closer to
/// the sink than its `from`. Pipe rates are unique from mass balance. Pressures
/// walk upstream from the fixed sales pressure through pipes and compressors.
pub fn solve_dry_gas_tree(problem: &DryGasProblem<'_>) -> Result<SteadyStateResult> {
    problem.topology.validate()?;
    if problem.sink_pressure_pa <= 0.0 {
        return Err(HydraulicsError::InvalidInput(
            "sink pressure must be positive".into(),
        ));
    }
    if problem.topology.node(&problem.sink_id).is_none() {
        return Err(HydraulicsError::UnknownSink(problem.sink_id.0.clone()));
    }
    for node_id in problem.injections_kg_s.keys() {
        if problem.topology.node(&NodeId(node_id.clone())).is_none() {
            return Err(HydraulicsError::UnknownInjection(node_id.clone()));
        }
    }

    let order = topological_upstream_order(problem.topology, &problem.sink_id)?;
    let edge_flow = assign_tree_flows(problem.topology, &problem.injections_kg_s, &problem.sink_id)?;

    let mut pressure: HashMap<String, f64> = HashMap::new();
    pressure.insert(problem.sink_id.0.clone(), problem.sink_pressure_pa);

    let mut edges_into: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in problem.topology.edges.iter().enumerate() {
        edges_into.entry(e.to.0.clone()).or_default().push(i);
    }

    for node in &order {
        let Some(incoming) = edges_into.get(node) else {
            continue;
        };
        let p_to = *pressure.get(node).ok_or_else(|| HydraulicsError::NotATree {
            sink: problem.sink_id.0.clone(),
            reason: format!("pressure missing at {node} while walking upstream"),
        })?;

        for &ei in incoming {
            let edge = &problem.topology.edges[ei];
            let flow = edge_flow.get(edge.id.as_str()).copied().unwrap_or(0.0);
            let p_from = match edge.kind {
                EdgeKind::Pipe | EdgeKind::Valve => {
                    let pipe = problem
                        .hydraulics
                        .pipes
                        .get(edge.id.as_str())
                        .ok_or_else(|| HydraulicsError::MissingPipe(edge.id.0.clone()))?;
                    isothermal_upstream_pressure(p_to, flow, pipe, &problem.hydraulics.gas)
                        .map_err(|err| HydraulicsError::UnsolvablePipe {
                            edge: edge.id.0.clone(),
                            flow_kg_s: flow,
                            reason: err.to_string(),
                        })?
                }
                EdgeKind::Compressor => {
                    let spec = problem
                        .hydraulics
                        .compressors
                        .get(edge.id.as_str())
                        .ok_or_else(|| HydraulicsError::MissingCompressor(edge.id.0.clone()))?;
                    let suction = p_to - spec.boost_pa;
                    if suction <= 0.0 {
                        return Err(HydraulicsError::CompressorSuctionNonPositive {
                            edge: edge.id.0.clone(),
                            suction_pa: suction,
                        });
                    }
                    suction
                }
                EdgeKind::Pump => {
                    return Err(HydraulicsError::InvalidInput(format!(
                        "edge {} is a pump; dry-gas solver expects compressor or pipe",
                        edge.id.as_str()
                    )));
                }
            };

            if let Some(existing) = pressure.get(edge.from.as_str()) {
                if (existing - p_from).abs() > 1.0 {
                    return Err(HydraulicsError::NotATree {
                        sink: problem.sink_id.0.clone(),
                        reason: format!(
                            "conflicting pressure at {} ({} vs {})",
                            edge.from.as_str(),
                            existing,
                            p_from
                        ),
                    });
                }
            } else {
                pressure.insert(edge.from.0.clone(), p_from);
            }
        }
    }

    for node in &problem.topology.nodes {
        if !pressure.contains_key(node.id.as_str()) {
            return Err(HydraulicsError::NotATree {
                sink: problem.sink_id.0.clone(),
                reason: format!("node {} is not connected to the sink", node.id.as_str()),
            });
        }
    }

    let mut edge_flows = BTreeMap::new();
    for edge in &problem.topology.edges {
        let flow = edge_flow.get(edge.id.as_str()).copied().unwrap_or(0.0);
        edge_flows.insert(
            edge.id.0.clone(),
            EdgeFlow {
                edge_id: edge.id.0.clone(),
                mass_flow_kg_s: flow,
                p_from_pa: pressure[edge.from.as_str()],
                p_to_pa: pressure[edge.to.as_str()],
            },
        );
    }

    let mut node_pressure_pa = BTreeMap::new();
    for (k, v) in &pressure {
        node_pressure_pa.insert(k.clone(), *v);
    }

    let mut constraint_margin_pa = BTreeMap::new();
    let mut limiting_node = None;
    let mut worst = 0.0_f64;
    let mut breached = false;
    for (node_id, limit) in &problem.hydraulics.pressure_limits {
        let p = *pressure.get(node_id).ok_or_else(|| {
            HydraulicsError::InvalidInput(format!("pressure limit on unknown node {node_id}"))
        })?;
        let margin = limit.max_pa - p;
        constraint_margin_pa.insert(node_id.clone(), margin);
        if margin < 0.0 {
            breached = true;
            if limiting_node.is_none() || margin < worst {
                worst = margin;
                limiting_node = Some(node_id.clone());
            }
        }
    }

    Ok(SteadyStateResult {
        status: if breached {
            ConstraintStatus::Breached
        } else {
            ConstraintStatus::Feasible
        },
        node_pressure_pa,
        edge_flows,
        constraint_margin_pa,
        limiting_node,
    })
}

/// Sink-first order over the reverse adjacency (sales → pads).
fn topological_upstream_order(topology: &NetworkModel, sink: &NodeId) -> Result<Vec<String>> {
    // Reverse adjacency: original from→to becomes to→from.
    let mut downstream_to_upstream: HashMap<String, Vec<String>> = HashMap::new();
    let mut rev_indegree: HashMap<String, usize> = topology
        .nodes
        .iter()
        .map(|n| (n.id.0.clone(), 0))
        .collect();

    for e in &topology.edges {
        downstream_to_upstream
            .entry(e.to.0.clone())
            .or_default()
            .push(e.from.0.clone());
        *rev_indegree.get_mut(e.from.as_str()).unwrap() += 1;
    }

    if *rev_indegree.get(sink.as_str()).unwrap_or(&1) != 0 {
        return Err(HydraulicsError::NotATree {
            sink: sink.0.clone(),
            reason: "sink is not a root of the reverse adjacency (check edge orientation)"
                .into(),
        });
    }

    let mut q = VecDeque::new();
    for (id, deg) in &rev_indegree {
        if *deg == 0 {
            q.push_back(id.clone());
        }
    }

    let mut order = Vec::new();
    let mut seen = HashSet::new();
    while let Some(n) = q.pop_front() {
        if !seen.insert(n.clone()) {
            continue;
        }
        order.push(n.clone());
        if let Some(upstreams) = downstream_to_upstream.get(&n) {
            for up in upstreams {
                let d = rev_indegree.get_mut(up.as_str()).unwrap();
                *d -= 1;
                if *d == 0 {
                    q.push_back(up.clone());
                }
            }
        }
    }

    if order.len() != topology.nodes.len() {
        return Err(HydraulicsError::NotATree {
            sink: sink.0.clone(),
            reason: format!(
                "cycle or disconnected graph (ordered {}, nodes {})",
                order.len(),
                topology.nodes.len()
            ),
        });
    }
    Ok(order)
}

fn assign_tree_flows(
    topology: &NetworkModel,
    injections: &BTreeMap<String, f64>,
    sink: &NodeId,
) -> Result<HashMap<String, f64>> {
    let mut children_edges: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, e) in topology.edges.iter().enumerate() {
        children_edges.entry(e.to.0.clone()).or_default().push(i);
    }

    fn outflow(
        node: &str,
        sink: &str,
        injections: &BTreeMap<String, f64>,
        children_edges: &HashMap<String, Vec<usize>>,
        topology: &NetworkModel,
        edge_flow: &mut HashMap<String, f64>,
        visiting: &mut HashSet<String>,
    ) -> Result<f64> {
        if !visiting.insert(node.to_string()) {
            return Err(HydraulicsError::NotATree {
                sink: sink.to_string(),
                reason: format!("cycle detected at {node}"),
            });
        }
        let mut total = injections.get(node).copied().unwrap_or(0.0);
        if let Some(idxs) = children_edges.get(node) {
            for &i in idxs {
                let edge = &topology.edges[i];
                let child_out = outflow(
                    edge.from.as_str(),
                    sink,
                    injections,
                    children_edges,
                    topology,
                    edge_flow,
                    visiting,
                )?;
                edge_flow.insert(edge.id.0.clone(), child_out);
                total += child_out;
            }
        }
        visiting.remove(node);
        Ok(total)
    }

    let mut edge_flow = HashMap::new();
    let mut visiting = HashSet::new();
    let _ = outflow(
        sink.as_str(),
        sink.as_str(),
        injections,
        &children_edges,
        topology,
        &mut edge_flow,
        &mut visiting,
    )?;

    if edge_flow.len() != topology.edges.len() {
        return Err(HydraulicsError::NotATree {
            sink: sink.0.clone(),
            reason: "not every edge received a mass-balance flow".into(),
        });
    }
    Ok(edge_flow)
}
