# landmark-network-core

Foundation crate for **Landmark Network Intelligence**: a persistent oil and gas
field network model with typed topology, content identity, and structural
validation.

This is the CYME-style *product pattern* applied to hydrocarbon assets — not a
utilities power-flow product. Physics, reliability, expansion, and economics
live in sibling crates that consume this model.

## Workspace

| Crate | Role |
|---|---|
| `network-core` | Nodes, edges, topology, identity, validation |
| `network-hydraulics` | Steady-state dry-gas tree solve; two-pad fixture |

Planned: `network-reliability`, `network-expand`, `network-economics`.

## Boundary

- **In (core):** graph topology, asset identity, fluid service, structural refusals
- **In (hydraulics):** isothermal dry-gas pressures/rates on a gathering tree
- **Out:** multiphase, looped networks, outage Monte Carlo, MIP expansion, HTTP

Segment friction comes from `landmark-pipeline-core` (`pipeline-physics`).
Field Network prices projects once hydraulics supply feasible capacities.

## Develop

```bash
cargo test --workspace
```

## First tie-in fixture

`network_hydraulics::two_pad_gathering` — two pads, shared trunk, compressor.
Each pad alone is feasible; both together breach a pressure ceiling; an upgraded
compressor clears the breach.

```bash
cargo test -p network-hydraulics
cargo run -p network-hydraulics --example dump_capacity
```

Capacity numbers feed `landmark-field-network` `examples.two_pad_tie_in`, which
enumerates pad A / pad B / both / both+upgrade and reports NPV plus compressor
capex breakeven.
