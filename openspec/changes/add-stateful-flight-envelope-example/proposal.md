## Why

The DeepCausality framework provides two distinct composition mechanisms that both
produce `CausalEffectPropagationProcess`: a `Causaloid` (rich isomorphic recursive
causal structure — singleton, collection, or hypergraph) and a `CausalMonad` (rigid
sequential functional composition via `bind`). Because both yield the same process
type, they are structurally composable end-to-end. No example in the repository
currently demonstrates this uniform composition, and no example exercises the full
stateful form `CausalEffectPropagationProcess<Value, State, Context, Error, Log>`
where `State` and `Context` are non-trivial. New users therefore have no concrete
reference for combining the two and for carrying real Markovian state across
heterogeneous causal stages.

## What Changes

- Add a new runnable example crate member `flight_envelope_monitor` under
  `examples/avionics_examples/`, alongside the existing `geometric_tcas`,
  `hypersonic_2t`, and `magnav` examples.
- The example composes three stages, each producing a
  `CausalEffectPropagationProcess<_, FlightState, AircraftConfig, CausalityError, EffectLog>`:
  1. A `Causaloid::from_causal_collection` of per-sensor singleton causaloids
     (airspeed, altitude, attitude, engine, fuel-flow) aggregated via
     `AggregateLogic` into a single `SensorHealth` verdict.
  2. A `CausalMonad` bind-chain that performs unit conversion, a one-step Kalman
     covariance update, and state estimation — purely sequential, with
     `FlightState` (estimate + covariance + accumulated risk) carried in the
     process `State` and `AircraftConfig` (mass, MTOW, stall margin, service
     ceiling) carried in the process `Context`.
  3. A `Causaloid::from_causal_graph_with_context` over a flight-envelope
     hypergraph (stall, overspeed, terrain proximity, traffic conflict, icing,
     CG-out-of-limits) producing a final `SafetyVerdict`.
- Demonstrate that `EffectLog` accumulates across all three stages and that an
  error in any stage short-circuits the rest while preserving logs.
- Register the new example in the `avionics_examples` Cargo workspace and Bazel
  build configuration following the convention used by the sibling examples.
- Add a README that explains why each stage uses the structure it does, what is
  carried in `State` vs `Context`, and how this example differs from the
  stateless `PropagatingEffect` form.

## Capabilities

### New Capabilities

- `flight-envelope-monitor-example`: A runnable avionics example that
  demonstrates uniform composition of a `Causaloid` collection, a `CausalMonad`
  bind-chain, and a `Causaloid` hypergraph through a single
  `CausalEffectPropagationProcess` with non-trivial `State` (Kalman covariance
  plus accumulated risk) and `Context` (aircraft configuration). The capability
  covers the example structure, the three composition stages, the state and
  context flow, the audit-log accumulation, and the error short-circuit
  behavior.

### Modified Capabilities

<!-- None. No existing openspec capability specs. No requirement changes to
existing crates; this change is purely additive in the examples tree. -->

## Impact

- **New code**: `examples/avionics_examples/flight_envelope_monitor/` with
  `main.rs`, `model.rs`, and `README.md`.
- **Modified code**:
  - `examples/avionics_examples/Cargo.toml` — register the new binary target.
  - `examples/avionics_examples/BUILD.bazel` — add the corresponding Bazel
    `rust_binary` target.
  - `examples/avionics_examples/README.md` — add a one-line entry pointing to
    the new example.
- **Dependencies**: Uses only `deep_causality` and `deep_causality_core` from
  the workspace. No new external crates are introduced (per AGENTS.md guidance).
- **APIs touched**: Consumes the existing public APIs of
  `CausalEffectPropagationProcess` (notably `with_state`, `bind`,
  `bind_or_error`, `from_value`, `from_error`), `CausalMonad`, `Causaloid`
  constructors (`new`, `new_with_context`, `from_causal_collection`,
  `from_causal_graph_with_context`), and `MonadicCausable::evaluate`. No public
  API changes are required.
- **Testing**: The example is runnable via `cargo run -p avionics_examples
  --bin flight_envelope_monitor` and is exercised by the existing
  `make build` / `make test` flows for the `avionics_examples` crate.
- **Risk**: Low. Purely additive example code in the examples tree; no library
  crate is modified.
