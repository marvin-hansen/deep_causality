## ADDED Requirements

### Requirement: Example crate layout

The repository SHALL contain a runnable example at
`examples/avionics_examples/flight_envelope_monitor/` consisting of `main.rs`,
`model.rs`, and `README.md`, registered as a binary target in both
`examples/avionics_examples/Cargo.toml` and
`examples/avionics_examples/BUILD.bazel`.

#### Scenario: Cargo build of the example crate succeeds

- **WHEN** a developer runs `cargo build -p avionics_examples` from the
  repository root
- **THEN** the workspace compiles without errors and the
  `flight_envelope_monitor` binary target is produced

#### Scenario: Bazel build of the example target succeeds

- **WHEN** a developer runs the Bazel target for `flight_envelope_monitor`
  declared in `examples/avionics_examples/BUILD.bazel`
- **THEN** the target builds without errors using the same dependency set as
  the sibling avionics examples

#### Scenario: Example crate is documented in the avionics README

- **WHEN** a reader opens `examples/avionics_examples/README.md`
- **THEN** the README lists `flight_envelope_monitor` alongside
  `geometric_tcas`, `hypersonic_2t`, and `magnav` with a one-line summary
  pointing to its README

### Requirement: Stateful process type

The example SHALL operate on
`CausalEffectPropagationProcess<Value, FlightState, AircraftConfig, CausalityError, EffectLog>`
where `FlightState` carries the four-element estimate vector, the
corresponding diagonal covariance, and an accumulated scalar risk; and where
`AircraftConfig` carries mass, maximum takeoff weight, stall-margin
multiplier, and service ceiling.

#### Scenario: State and Context types are non-trivial

- **WHEN** the source of `model.rs` is inspected
- **THEN** neither `FlightState` nor `AircraftConfig` is `()` and both are
  defined as named structs with the documented fields

#### Scenario: Process type is the full form, not the stateless alias

- **WHEN** the type produced by each of the three stages is inspected
- **THEN** the type is
  `CausalEffectPropagationProcess<_, FlightState, AircraftConfig, CausalityError, EffectLog>`
  and is not `PropagatingEffect<_>`

### Requirement: Three-stage uniform composition

The example SHALL execute exactly three composition stages whose outputs flow
end-to-end through the same process type without manual unwrapping:

1. A `Causaloid` collection of per-sensor singleton causaloids, constructed
   via `Causaloid::from_causal_collection_with_context`, aggregating sensor
   readings via `AggregateLogic` to produce a `SensorHealth` verdict.
2. A `CausalMonad` bind-chain of at least three steps (validation, Kalman
   covariance update, state estimation) producing a `FlightStateEstimate`.
3. A `Causaloid` hypergraph of envelope-violation nodes, constructed via
   `Causaloid::from_causal_graph_with_context`, producing a final
   `SafetyVerdict`.

#### Scenario: Stage 1 is a Causaloid collection

- **WHEN** the construction of the sensor stage is inspected
- **THEN** it is a `Causaloid` of `CausaloidType::Collection` containing at
  least five child singleton causaloids and uses an `AggregateLogic` with a
  numeric threshold

#### Scenario: Stage 2 is a CausalMonad bind-chain

- **WHEN** the middle stage is inspected
- **THEN** it consists of at least three sequential `bind` (or `bind_or_error`)
  calls on a `CausalEffectPropagationProcess`, with no use of any
  `Causaloid` between the stage's first and last `bind`

#### Scenario: Stage 3 is a Causaloid hypergraph

- **WHEN** the envelope stage is inspected
- **THEN** it is a `Causaloid` of `CausaloidType::Graph` constructed via
  `Causaloid::from_causal_graph_with_context` over a `CausaloidGraph`
  containing at least five nodes and at least one edge per node

#### Scenario: Stages are connected without manual unwrapping

- **WHEN** the connections between the three stages are inspected
- **THEN** the output of stage N is passed directly into stage N+1 by reading
  the `CausalEffectPropagationProcess` returned by stage N (not by
  destructuring `EffectValue`, error fields, or logs by hand)

### Requirement: State evolution across stages

The example SHALL evolve `FlightState` across the bind-chain such that the
covariance after the Kalman step differs from the covariance before it, and
the accumulated risk after the envelope stage reflects contributions from
both the sensor stage and the envelope stage.

#### Scenario: Kalman step changes covariance

- **WHEN** the example runs the nominal scenario
- **THEN** the `covariance` field of `FlightState` after the Kalman step is
  numerically different from the `covariance` field before it

#### Scenario: Risk accumulates across stages

- **WHEN** the example runs the nominal scenario and the final
  `FlightState.risk` is observed
- **THEN** the value reflects contributions from both the sensor-aggregation
  stage and the envelope-evaluation stage, with each contribution traceable
  in `EffectLog`

### Requirement: Audit-log accumulation

The example SHALL accumulate `EffectLog` entries across all three stages such
that the final `EffectLog` contains entries originating from every stage in
the order they were executed.

#### Scenario: Final log contains entries from every stage

- **WHEN** the example runs and the final `EffectLog` is inspected
- **THEN** the log contains at least one entry attributable to the sensor
  collection stage, at least one entry attributable to each step of the bind
  chain, and at least one entry attributable to the envelope hypergraph
  stage, in chronological order

### Requirement: Error short-circuit semantics

The example SHALL demonstrate that an error produced in any stage prevents
execution of subsequent stages while preserving the `EffectLog` accumulated
up to and including the failing stage.

#### Scenario: Failing sensor scenario short-circuits the bind-chain and the hypergraph

- **WHEN** the example runs the failing-sensor scenario in `main.rs`
- **THEN** the final process carries `error: Some(CausalityError(...))`, the
  bind-chain steps after the failing sensor stage and the envelope stage do
  not execute, and the final `EffectLog` still contains the entries produced
  before and including the failing stage

#### Scenario: Final state on error reflects the last successful stage

- **WHEN** the failing-sensor scenario completes
- **THEN** the `state` field of the returned process is the `FlightState`
  carried at the moment of failure (not a freshly defaulted `FlightState`)

### Requirement: No new external dependencies

The example SHALL depend only on `deep_causality` and `deep_causality_core`
from the workspace; no new external crates SHALL be added to
`examples/avionics_examples/Cargo.toml` for this example.

#### Scenario: Cargo manifest adds no new external crate

- **WHEN** the diff to `examples/avionics_examples/Cargo.toml` is inspected
- **THEN** the only added entries are the new binary target stanza and (if
  needed) workspace dependency lines for `deep_causality` and
  `deep_causality_core`; no new entries from crates.io are introduced

### Requirement: Two demonstration scenarios in main

The example's `main.rs` SHALL run two named scenarios in sequence: a nominal
scenario that produces a `SafetyVerdict` and a failing-sensor scenario that
produces an error, printing for each scenario the final verdict (or error),
the final `FlightState`, and the full `EffectLog`.

#### Scenario: Nominal run prints a verdict and a non-empty log

- **WHEN** the binary is invoked via `cargo run -p avionics_examples --bin flight_envelope_monitor`
- **THEN** the output contains a section labeled "Nominal" that prints a
  non-error `SafetyVerdict` and a non-empty `EffectLog` originating from all
  three stages

#### Scenario: Failing-sensor run prints an error and a non-empty log

- **WHEN** the binary is invoked via `cargo run -p avionics_examples --bin flight_envelope_monitor`
- **THEN** the output contains a section labeled "Failing sensor" that prints
  the `CausalityError` carried by the final process and a non-empty
  `EffectLog` covering the stages executed before the failure
