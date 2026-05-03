## ADDED Requirements

### Requirement: Example crate layout

The repository SHALL contain a runnable example at
`examples/avionics_examples/flight_envelope_monitor/` consisting of `main.rs`,
`model.rs`, and `README.md`, registered as an `[[example]]` target in
`examples/avionics_examples/Cargo.toml` with `name = "flight_envelope_monitor"`
and `path = "flight_envelope_monitor/main.rs"`. No Bazel `BUILD.bazel` file is
added under the examples tree.

#### Scenario: Cargo build of the example target succeeds

- **WHEN** a developer runs `cargo build -p avionics_examples` from the
  repository root
- **THEN** the workspace compiles without errors and the
  `flight_envelope_monitor` example target is produced

#### Scenario: Example target is registered alongside the siblings

- **WHEN** the diff to `examples/avionics_examples/Cargo.toml` is inspected
- **THEN** the new `[[example]]` stanza for `flight_envelope_monitor` is
  present alongside the existing `magnav`, `geometric_tcas`, and
  `hypersonic_2t` stanzas, all using the `[[example]]` (not `[[bin]]`) form

#### Scenario: Example crate is documented in the avionics README

- **WHEN** a reader opens `examples/avionics_examples/README.md`
- **THEN** the README lists `flight_envelope_monitor` alongside
  `geometric_tcas`, `hypersonic_2t`, and `magnav` with a one-line summary
  pointing to its README

### Requirement: Stateful process type

All three composition stages of the example SHALL produce values typed as
`PropagatingProcess<_, FlightState, AircraftConfig>` (the public alias of
`CausalEffectPropagationProcess<_, FlightState, AircraftConfig, CausalityError, EffectLog>`),
where:

- `FlightState` is a struct with fields `estimate: [f64; 4]`,
  `covariance: [f64; 4]`, and `risk: f64`, deriving `Default + Clone + Debug`.
- `AircraftConfig` is a struct with fields `mass_kg: f64`, `mtow_kg: f64`,
  `stall_margin: f64`, and `service_ceiling_m: f64`, deriving `Clone + Debug`.

Neither `FlightState` nor `AircraftConfig` is `()`; both are non-trivial named
structs.

#### Scenario: State and Context types are non-trivial

- **WHEN** the source of `model.rs` is inspected
- **THEN** `FlightState` and `AircraftConfig` are defined as named structs with
  the documented fields and their derived traits include the documented set

#### Scenario: Each stage's output type is a stateful process

- **WHEN** the type produced by each of the three stages is inspected
- **THEN** the type is
  `PropagatingProcess<_, FlightState, AircraftConfig>` (or its long-form
  equivalent `CausalEffectPropagationProcess<_, FlightState, AircraftConfig, CausalityError, EffectLog>`),
  not `PropagatingEffect<_>`

### Requirement: Three-stage uniform composition

The example SHALL execute exactly three composition stages, connected
end-to-end by passing each stage's returned `PropagatingProcess` to the next
without manual unwrapping of `EffectValue`, `state`, `error`, or `logs`:

1. A `Causaloid` collection of five per-sensor singleton causaloids,
   constructed via `Causaloid::from_causal_collection_with_context`, evaluated
   via `StatefulMonadicCausableCollection::evaluate_collection_stateful`,
   producing a `PropagatingProcess<f64, FlightState, AircraftConfig>` whose
   value channel carries the joint health probability.
2. A `CausalMonad` `bind`-chain of three steps (health-fold, Kalman update,
   estimate write) operating on the stateful `PropagatingProcess` returned by
   Stage 1, producing a `PropagatingProcess<FlightStateEstimate, FlightState, AircraftConfig>`.
3. A `Causaloid` hypergraph of envelope-violation nodes, constructed via
   `Causaloid::from_causal_graph_with_context`, evaluated via
   `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`
   on a `CausaloidGraph<Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>>`,
   producing a `PropagatingProcess<FlightStateEstimate, FlightState, AircraftConfig>`.

#### Scenario: Stage 1 is a Causaloid collection of five sensors

- **WHEN** the construction of the sensor stage is inspected
- **THEN** it is a `Causaloid` of `CausaloidType::Collection` containing five
  child singleton causaloids (one per sensor: airspeed, altitude, attitude,
  vertical-speed, fuel-flow) wrapped via
  `Causaloid::from_causal_collection_with_context`, and is evaluated via
  `StatefulMonadicCausableCollection::evaluate_collection_stateful`

#### Scenario: Stage 1 aggregates per-sensor f64 health via AggregateLogic::All

- **WHEN** the collection construction is inspected
- **THEN** the `AggregateLogic` argument is `AggregateLogic::All` and the
  `threshold_value` argument is irrelevant for f64 (the framework's
  `Aggregatable for f64` impl ignores `threshold` and computes `∏ p_i` for
  `All`)

#### Scenario: Stage 2 is a CausalMonad bind-chain of three steps

- **WHEN** the middle stage is inspected
- **THEN** it consists of exactly three sequential `bind` (or `bind_or_error`)
  calls on a `PropagatingProcess`, with no use of any `Causaloid` between the
  first and last `bind`

#### Scenario: Stage 3 is a Causaloid hypergraph of six envelope nodes

- **WHEN** the envelope stage is inspected
- **THEN** it is a `Causaloid` of `CausaloidType::Graph` constructed via
  `Causaloid::from_causal_graph_with_context` over a `CausaloidGraph` containing
  exactly six nodes (`stall_risk`, `overspeed_risk`, `terrain_proximity`,
  `traffic_conflict`, `icing_risk`, `cg_out_of_limits`) with at least one edge
  per node, and is evaluated via
  `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`

#### Scenario: Stages are connected without manual unwrapping

- **WHEN** the connections between the three stages are inspected
- **THEN** the output of stage N is passed directly into stage N+1 by reading
  the `PropagatingProcess` returned by stage N (not by destructuring
  `EffectValue`, error fields, or logs by hand)

### Requirement: Per-sensor health probability convention

Each per-sensor causaloid in Stage 1 SHALL emit an `f64 ∈ [0.0, 1.0]`
representing that sensor's health probability (1.0 = perfectly healthy,
0.0 = fully degraded). The collection's joint health probability is the
product `∏ p_i` produced by `AggregateLogic::All`.

#### Scenario: Per-sensor closure returns a normalised health probability

- **WHEN** a sensor closure body is inspected
- **THEN** the returned `EffectValue<f64>` carries a value in the range
  `[0.0, 1.0]` (clamped if needed), computed from the sensor field's deviation
  from a healthy band defined in `model.rs`

#### Scenario: Joint health probability degrades smoothly when sensors degrade

- **WHEN** a `SensorReading` is constructed where one sensor field deviates
  from its healthy band by a measurable amount and the other four sensors
  remain at perfect health
- **THEN** the value channel of the Stage 1 output is strictly less than 1.0,
  strictly greater than 0.0, and equals the per-sensor health value of the
  degraded sensor (within floating-point tolerance), demonstrating the smooth
  product-aggregation behaviour

### Requirement: State evolution across stages

The example SHALL evolve `FlightState` across the three stages such that:

- `state.covariance` after the Kalman step in Stage 2 differs numerically from
  `state.covariance` before it.
- `state.risk` after the full pipeline reflects contributions from both the
  bind-chain's health-fold step (Stage 2 first bind) and from at least one
  envelope node in Stage 3, with each contribution traceable in `EffectLog`.
- `state.estimate` after Stage 2's third bind step differs from the input
  estimate vector.

#### Scenario: Kalman step changes covariance

- **WHEN** the example runs the nominal scenario
- **THEN** the `covariance` field of `FlightState` after the Kalman step is
  numerically different from the `covariance` field before it

#### Scenario: Risk accumulates from both Stage 2 and Stage 3

- **WHEN** the example runs the nominal scenario where Stage 1 produces an
  aggregated health less than 1.0 AND at least one Stage-3 envelope node
  contributes a non-zero risk increment
- **THEN** the final `state.risk` is strictly greater than the sum of either
  stage's contribution alone, and `EffectLog` contains entries naming the
  health-fold step (Stage 2) AND at least one envelope node (Stage 3)

### Requirement: SafetyVerdict derived from final state

The `SafetyVerdict` (`Nominal | Caution | Warning | Failure`) SHALL be derived
in `main.rs` from the final `state.risk` after all three stages have run, using
fixed thresholds documented in `model.rs`. The Stage-3 graph SHALL NOT
transmute its value channel into a `SafetyVerdict`; the value channel
`V = FlightStateEstimate` is preserved end-to-end through the graph (consistent
with the `StatefulMonadicCausableGraphReasoning<V, S, C>` constraint that the
Causaloid input value type equals the output value type).

#### Scenario: Verdict derived from final risk score

- **WHEN** the source of `main.rs` is inspected
- **THEN** the verdict is computed by thresholding `final_process.state.risk`
  against the documented `SafetyVerdict` thresholds; it is NOT extracted from
  `final_process.value`

#### Scenario: Graph value channel preserves FlightStateEstimate

- **WHEN** the type of the `Causaloid` instances stored in the graph is
  inspected
- **THEN** each node has type
  `Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>`
  (input and output value type identical)

### Requirement: Audit-log accumulation

The final `EffectLog` returned at the end of the pipeline SHALL contain log
entries originating from every stage in the order they were executed: at least
one entry from the sensor collection stage, at least one entry from each of
the three bind-chain steps, and at least one entry from the envelope hypergraph
stage.

#### Scenario: Final log contains entries from every stage in chronological order

- **WHEN** the example runs and the final `EffectLog` is inspected
- **THEN** the log contains the documented entries from every stage in
  chronological order, traceable to specific causaloid IDs (singleton sensor
  ids, envelope node ids) and bind-step labels

### Requirement: Error short-circuit semantics

The example SHALL demonstrate that an error produced in any stage prevents
execution of subsequent stages while preserving the `EffectLog` accumulated
up to and including the failing stage and the `state` carried at the moment
of failure.

#### Scenario: Failing-sensor scenario short-circuits the bind-chain and the hypergraph

- **WHEN** the example runs the failing-sensor scenario in `main.rs` (a sensor
  closure deliberately returns `error: Some(CausalityError(...))`)
- **THEN** the final `PropagatingProcess` carries `error: Some(CausalityError(...))`,
  the bind-chain steps after Stage 1 and the envelope stage do not execute,
  and the final `EffectLog` contains entries produced before and including the
  failing stage but NO entries from the bind-chain or the envelope graph

#### Scenario: Final state on error reflects the moment of failure

- **WHEN** the failing-sensor scenario completes
- **THEN** `final_process.state` equals the `FlightState` carried at the
  moment the failing sensor's closure returned its error (not a freshly
  defaulted `FlightState`)

### Requirement: Cargo manifest changes

`examples/avionics_examples/Cargo.toml` SHALL gain (a) a new `[[example]]`
target stanza for `flight_envelope_monitor`, and (b) a workspace-internal path
dependency on `deep_causality`. No new entries from crates.io SHALL be
introduced.

#### Scenario: Manifest gains an [[example]] stanza and a deep_causality dependency

- **WHEN** the diff to `examples/avionics_examples/Cargo.toml` is inspected
- **THEN** the only added entries are (a) the `[[example]]` stanza for
  `flight_envelope_monitor` and (b) `deep_causality = { path = "../../deep_causality" }`
  under `[dependencies]`; no `crates.io` entries are introduced

### Requirement: Two demonstration scenarios in main

The example's `main.rs` SHALL run two named scenarios in sequence: a nominal
scenario that produces a non-error `SafetyVerdict` and a failing-sensor
scenario that produces an error, printing for each scenario the verdict (or
error), the final `FlightState`, and the full `EffectLog`. The program SHALL
exit with status zero in both scenarios; the failing scenario surfaces its
error through stdout, not via `std::process::exit`.

#### Scenario: Nominal run prints a verdict and a non-empty log

- **WHEN** the binary is invoked via
  `cargo run -p avionics_examples --example flight_envelope_monitor`
- **THEN** the output contains a section labeled "Nominal" that prints a
  non-error `SafetyVerdict`, the final `FlightState` with non-zero `risk`, and
  a non-empty `EffectLog` originating from all three stages

#### Scenario: Failing-sensor run prints an error and a partial log

- **WHEN** the binary is invoked via
  `cargo run -p avionics_examples --example flight_envelope_monitor`
- **THEN** the output contains a section labeled "Failing sensor" that prints
  the `CausalityError` carried by the final process, the `FlightState` at the
  moment of failure, and an `EffectLog` covering the stages executed before
  the failure but no bind-chain or envelope-stage entries

#### Scenario: Both scenarios exit with status zero

- **WHEN** the binary is invoked via
  `cargo run -p avionics_examples --example flight_envelope_monitor`
- **THEN** the process exits with status code `0` regardless of the
  failing-sensor scenario's reported error
