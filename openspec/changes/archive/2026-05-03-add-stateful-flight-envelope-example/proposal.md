## Why

The DeepCausality framework now exposes (post `add-stateful-causaloid-evaluation`) a
unified stateful evaluation surface across all three causal forms — singleton via
`StatefulMonadicCausable::evaluate_stateful`, collection via
`StatefulMonadicCausableCollection::evaluate_collection_stateful`, and graph via
`StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`. All
three return `PropagatingProcess<O, S, C>` directly, threading user-defined `S`
(state) and `C` (context) end-to-end.

No example in the repository demonstrates this uniform composition with non-trivial
state and context. Without a runnable reference, new users have no concrete model
for combining a `Causaloid` collection, a `CausalMonad` `bind`-chain, and a
`Causaloid` hypergraph through a single stateful process — the central pedagogical
gap that motivates this change.

## What Changes

- Add a new runnable example crate member `flight_envelope_monitor` under
  `examples/avionics_examples/`, alongside the existing `geometric_tcas`,
  `hypersonic_2t`, and `magnav` examples. Registered as an `[[example]]` target
  (matching the sibling convention), invoked via
  `cargo run -p avionics_examples --example flight_envelope_monitor`.
- The example composes three stages, each producing a
  `PropagatingProcess<_, FlightState, AircraftConfig>`:
  1. A `Causaloid` collection of five per-sensor singleton causaloids (airspeed,
     altitude, attitude, vertical-speed, fuel-flow) where each sensor closure
     returns an `f64 ∈ [0.0, 1.0]` representing per-sensor health probability.
     Aggregation via `AggregateLogic::All` yields a joint health probability —
     the product of per-sensor healths. This produces a smooth deterioration
     signal: as any one sensor degrades, the aggregated health drops
     continuously rather than discretely tripping. Evaluated via
     `StatefulMonadicCausableCollection::evaluate_collection_stateful`.
  2. A `CausalMonad` `bind`-chain of three steps. The first step folds the
     joint health probability into `state.risk` as a contribution proportional
     to `1.0 - aggregated_health`, and converts the value channel to
     `FlightStateEstimate`. The second step performs a one-iteration scalar
     Kalman covariance update on `state.covariance`. The third step writes the
     estimated state into `state.estimate`. The bind-chain operates on the
     stateful `PropagatingProcess` returned by Stage 1 — no manual unwrapping.
  3. A `Causaloid` hypergraph of six envelope-violation nodes (stall, overspeed,
     terrain proximity, traffic conflict, icing, CG-out-of-limits). The graph
     value channel is `V = FlightStateEstimate` end-to-end (the graph reasoning
     trait constrains input value type equal to output value type); each node
     reads `FlightStateEstimate` from the value channel and `AircraftConfig`
     from `Context`, then accumulates an envelope-specific risk increment into
     `state.risk`. Evaluated via
     `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`
     on a `CausaloidGraph<Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>>`.
- The final `SafetyVerdict` (`Nominal | Caution | Warning | Failure`) is derived
  in `main.rs` from the final `state.risk` after all three stages have run. The
  graph value channel does **not** carry the verdict; the State channel is the
  authoritative risk record.
- Demonstrate that `EffectLog` accumulates across all three stages and that an
  error in any stage short-circuits the rest while preserving accumulated logs
  and the `state` at the moment of failure.
- Register the new example in the `avionics_examples` Cargo manifest. Add
  `deep_causality` as a workspace dependency on the manifest (it currently
  depends on `deep_causality_core`, `deep_causality_tensor`, and
  `deep_causality_multivector`, but not `deep_causality` — which is required for
  `Causaloid`, `CausaloidGraph`, the new stateful traits, and `AggregateLogic`).
- Add a README that explains why each stage uses the structure it does, what
  flows through the **value channel** vs the **State channel**, why per-sensor
  f64 with `AggregateLogic::All` produces a smooth deterioration signal, and
  how the example differs from a stateless `PropagatingEffect`-based
  composition.

## Capabilities

### New Capabilities

- `flight-envelope-monitor-example`: A runnable avionics example that
  demonstrates uniform composition of a `Causaloid` collection, a `CausalMonad`
  `bind`-chain, and a `Causaloid` hypergraph through a single stateful
  `PropagatingProcess` with non-trivial `State` (Kalman covariance + accumulated
  risk + estimate vector) and `Context` (aircraft configuration). The capability
  covers the example structure, the three composition stages, the value-channel
  vs State-channel separation, the f64-health-probability convention, the
  audit-log accumulation, the error short-circuit behaviour, and the
  SafetyVerdict-from-state derivation.

### Modified Capabilities

<!-- None. No requirement changes to existing crates; this change is purely
additive in the examples tree and consumes only public APIs. -->

## Impact

- **New code**: `examples/avionics_examples/flight_envelope_monitor/` with
  `main.rs`, `model.rs`, and `README.md`.
- **Modified code**:
  - `examples/avionics_examples/Cargo.toml` — add `[[example]]` target stanza
    for `flight_envelope_monitor`, and add `deep_causality = { path = "../../deep_causality" }`
    to `[dependencies]` (currently absent).
  - `examples/avionics_examples/README.md` — add a one-line entry pointing to
    the new example's README.
- **Bazel**: none. Examples have no `BUILD.bazel`; the avionics tree is
  Cargo-only.
- **Dependencies added to the workspace**: none. `deep_causality` is already a
  workspace member; we only add a workspace-internal path dependency from
  `avionics_examples` to it.
- **Public APIs consumed (all already exposed)**:
  - `Causaloid::new`, `Causaloid::new_with_context`,
    `Causaloid::from_causal_collection_with_context`,
    `Causaloid::from_causal_graph_with_context`.
  - `StatefulMonadicCausable::evaluate_stateful`,
    `StatefulMonadicCausableCollection::evaluate_collection_stateful`,
    `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`.
  - `PropagatingProcess<T, S, C>` (the public alias of
    `CausalEffectPropagationProcess`).
  - `CausalEffectPropagationProcess::bind` and `bind_or_error` for the
    `CausalMonad` middle stage.
  - `CausalMonad::pure` for the bind-chain entry.
  - `AggregateLogic::All`, `EffectValue`, `EffectLog`, `CausalityError`,
    `CausalityErrorEnum`, `LogAddEntry`.
  - `CausaloidGraph::add_root_causaloid`, `add_causaloid`, `add_edge`, `freeze`.
  - The `StatefulContextualCausalFn<I, O, S, C>` type alias for closure-author
    clarity.
- **No public API changes** are required.
- **Testing**: The example is runnable via `cargo run -p avionics_examples
  --example flight_envelope_monitor` and is exercised by the existing
  `make build` flow. No tests are added in the examples tree (per AGENTS.md
  guidance — examples are runnable demonstrations, not test fixtures).
- **Risk**: Low. Purely additive example code in the examples tree; no library
  crate is modified.
