## 1. Scaffolding

- [x] 1.1 Create directory `examples/avionics_examples/flight_envelope_monitor/`
- [x] 1.2 Create empty `main.rs`, `model.rs`, and `README.md` files in that directory
- [x] 1.3 Add an `[[example]]` target stanza named `flight_envelope_monitor` to `examples/avionics_examples/Cargo.toml` pointing at `flight_envelope_monitor/main.rs`, mirroring the existing entries for `geometric_tcas`, `hypersonic_2t`, and `magnav` (use `[[example]]` not `[[bin]]`)
- [x] 1.4 Add `deep_causality = { path = "../../deep_causality" }` to the `[dependencies]` of `examples/avionics_examples/Cargo.toml`. The manifest currently depends on `_core`, `_tensor`, `_multivector` but NOT `deep_causality`; the new example needs `Causaloid`, `CausaloidGraph`, `AggregateLogic`, and the new `StatefulMonadicCausable*` traits, all of which live in `deep_causality`.
- [x] 1.5 Append a one-line entry for `flight_envelope_monitor` to `examples/avionics_examples/README.md` mirroring the format of the existing entries
- [x] 1.6 Verify `cargo build -p avionics_examples` succeeds with the empty files in place

## 2. Domain types in `model.rs`

- [x] 2.1 Define `pub struct FlightState { pub estimate: [f64; 4], pub covariance: [f64; 4], pub risk: f64 }` deriving `Default + Clone + Debug` (the framework's stateful trait bounds require `Debug` for log formatting)
- [x] 2.2 Define `pub struct AircraftConfig { pub mass_kg: f64, pub mtow_kg: f64, pub stall_margin: f64, pub service_ceiling_m: f64 }` deriving `Clone + Debug`
- [x] 2.3 Define `pub struct SensorReading { pub airspeed_kn: f64, pub altitude_ft: f64, pub attitude_deg: f64, pub vertical_speed_fpm: f64, pub fuel_flow_pph: f64 }` deriving `Default + Clone + Debug`
- [x] 2.4 Define `pub struct FlightStateEstimate { pub airspeed_kn: f64, pub altitude_ft: f64, pub attitude_deg: f64, pub vertical_speed_fpm: f64 }` deriving `Default + Clone + Debug` — this is the value-channel type carried by the bind chain output and through the Stage-3 graph (V_in == V_out)
- [x] 2.5 Define `pub enum SafetyVerdict { Nominal, Caution, Warning, Failure }` deriving `Clone + Debug`. Document its risk-threshold mapping in a doc comment
- [x] 2.6 Define a local type alias `pub type FlightProcess<T> = PropagatingProcess<T, FlightState, AircraftConfig>;` for readability
- [x] 2.7 Define `pub struct HealthyBands { airspeed_kn: (f64, f64), altitude_ft: (f64, f64), attitude_deg: (f64, f64), vertical_speed_fpm: (f64, f64), fuel_flow_pph: (f64, f64) }` and a single `const NOMINAL_BANDS: HealthyBands` to anchor per-sensor health-probability calculations

## 3. Stage 1 — sensor causaloid collection (per-sensor f64 health)

- [x] 3.1 Implement five singleton `Causaloid<SensorReading, f64, FlightState, AircraftConfig>` constructors via `Causaloid::new` (stateless closure variant — per-sensor closures don't need state). Each closure: extract its field from `SensorReading`, compute `health = 1.0 - clamp(deviation_from_band / tolerance, 0.0, 1.0)`, return `PropagatingEffect::from_value(health)` with a log entry that names the sensor
- [x] 3.2 For the failing-sensor scenario, provide a sixth constructor variant `airspeed_failing(...)` whose closure returns `PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom("airspeed sensor lost".into())))`. This is the closure swapped in for the failing scenario; the nominal scenario uses the healthy `airspeed` closure
- [x] 3.3 Implement a builder function `pub fn build_sensor_collection(config: AircraftConfig, failing_airspeed: bool) -> Causaloid<SensorReading, f64, FlightState, AircraftConfig>` that wraps the appropriate five sensors via `Causaloid::from_causal_collection_with_context`, using `AggregateLogic::All` (NOT `Some(k)`) and `Some(0.0)` for the threshold value (irrelevant for f64 but the parameter must be supplied)
- [x] 3.4 Verify (by running the example) that with a nominal `SensorReading`, evaluating the collection's underlying `Vec` slice via `evaluate_collection_stateful` produces a non-error `FlightProcess<f64>` whose value is `1.0` (within fp tolerance) and whose `state` matches the input `FlightState::default()`

## 4. Stage 2 — `CausalMonad` `bind`-chain (three steps)

- [x] 4.1 Implement `health_fold_step(value: EffectValue<f64>, state: FlightState, ctx: Option<AircraftConfig>) -> FlightProcess<FlightStateEstimate>` as the FIRST bind step. Behaviour: extract `health` from the value, compute `state.risk += (1.0 - health) * 1.0` (weight = 1.0), then build a `FlightStateEstimate` from the incoming reading via the context (read aircraft config for any normalisation needed) and emit it on the value channel. Add a log entry "stage2.health_fold: risk += (1.0-health)"
- [x] 4.2 Implement `kalman_step(value, state, ctx) -> FlightProcess<FlightStateEstimate>` as the SECOND bind step. Behaviour: update each diagonal element of `state.covariance` via a one-iteration scalar Kalman update (e.g. `cov_i = cov_i * (1.0 - K_i)` where `K_i = cov_i / (cov_i + R)` for fixed measurement noise `R = 1.0`). Emit the same `FlightStateEstimate` on the value channel. Add a log entry "stage2.kalman: covariance updated"
- [x] 4.3 Implement `estimate_step(value, state, ctx) -> FlightProcess<FlightStateEstimate>` as the THIRD bind step. Behaviour: write the four `FlightStateEstimate` fields into `state.estimate` (`[airspeed_kn, altitude_ft, attitude_deg, vertical_speed_fpm]`), pass through the value unchanged. Add a log entry "stage2.estimate: state.estimate written"
- [x] 4.4 Compose the three steps via `bind_or_error` (use `bind_or_error` because each step requires the value to be `Some`; the helper short-circuits on `EffectValue::None`). Apply the chain directly to the `FlightProcess<f64>` returned by Stage 1 — no `with_state` lift is needed because Stage 1 already returns the stateful process type
- [x] 4.5 Verify the chain on a nominal input produces a non-error `FlightProcess<FlightStateEstimate>` whose `state.covariance` differs from the input covariance, whose `state.estimate` differs from the input estimate, whose `state.risk` is non-zero, and whose `EffectLog` contains the three step labels

## 5. Stage 3 — envelope causaloid hypergraph (V = FlightStateEstimate)

- [x] 5.1 Implement six singleton context-aware causaloids — `stall_risk`, `overspeed_risk`, `terrain_proximity`, `traffic_conflict`, `icing_risk`, `cg_out_of_limits` — each via `Causaloid::new_with_context` with closures of type `StatefulContextualCausalFn<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>`. Each closure: read `value` (the `FlightStateEstimate`) and `context` (the `AircraftConfig`), compute the node's risk increment, mutate `state.risk += increment`, and emit the same `FlightStateEstimate` unchanged on the value channel. Add a log entry naming the node
- [x] 5.2 Build a `CausaloidGraph<Causaloid<FlightStateEstimate, FlightStateEstimate, FlightState, AircraftConfig>>` that wires the six nodes with cross-edges: `stall_risk → terrain_proximity`, `icing_risk → stall_risk`, `traffic_conflict → overspeed_risk`, plus enough edges to ensure every node has at least one outgoing or incoming edge. Document the topology in a comment
- [x] 5.3 Wrap the graph via `Causaloid::from_causal_graph_with_context` with the supplied `AircraftConfig`. Note: this returns a `Causaloid` of `CausaloidType::Graph`, but for stateful graph evaluation the example invokes `evaluate_subgraph_from_cause_stateful` directly on the underlying `CausaloidGraph` (not on the wrapper Causaloid), starting from the root index 0
- [x] 5.4 Verify that evaluating the graph from index 0 on a nominal `FlightProcess<FlightStateEstimate>` produces a non-error `FlightProcess<FlightStateEstimate>` whose `state.risk` reflects increments from at least three of the six envelope nodes (BFS reaches them) and whose log includes envelope-stage entries

## 6. Demonstration in `main.rs`

- [x] 6.1 Construct a nominal `AircraftConfig` (e.g. mass=70_000 kg, MTOW=80_000 kg, stall_margin=1.3, service_ceiling=42_000 m)
- [x] 6.2 Construct a nominal `SensorReading` (e.g. airspeed=250 kn, altitude=10_000 ft, attitude=2 deg, vsi=0 fpm, fuel_flow=2400 pph) and a slightly-degraded `SensorReading` for the nominal scenario (one sensor at 90% health to give a non-zero `state.risk` contribution)
- [x] 6.3 Compose the three stages: build the sensor collection, evaluate it on the slightly-degraded reading via `evaluate_collection_stateful` (passing an initial `FlightProcess<SensorReading>` with `state: FlightState::default()`, `context: Some(config.clone())`), run the bind chain, then call `evaluate_subgraph_from_cause_stateful(0, &process)` on the envelope graph
- [x] 6.4 Derive the `SafetyVerdict` in `main.rs` by thresholding `final_process.state.risk` (e.g. `risk < 0.1 => Nominal`, `< 0.5 => Caution`, `< 1.0 => Warning`, else `Failure`). Document the thresholds in `model.rs`
- [x] 6.5 Print a section labeled "Nominal" containing the derived `SafetyVerdict`, the final `FlightState` (estimate, covariance, risk), and the full `EffectLog`
- [x] 6.6 Construct a failing scenario: same nominal `SensorReading` but build the sensor collection with `failing_airspeed = true` (the airspeed closure deliberately returns an error)
- [x] 6.7 Run the same composition on the failing collection. Print a section labeled "Failing sensor" containing the `CausalityError` from `final_process.error`, the `FlightState` at the moment of failure, and the partial `EffectLog`
- [x] 6.8 Verify the program exits with status zero in both scenarios — the failing scenario surfaces its error through stdout, not via `std::process::exit`

## 7. README

- [x] 7.1 In `examples/avionics_examples/flight_envelope_monitor/README.md`, write an opening paragraph explaining the **two channels** of `PropagatingProcess<T, FlightState, AircraftConfig>` — the **value channel** (`T` transitions across stages: `SensorReading → f64 → FlightStateEstimate → FlightStateEstimate`) and the **State channel** (`FlightState` accumulates across stages, carrying covariance and the cumulative risk)
- [x] 7.2 Document the three-stage pipeline with a small ASCII diagram showing collection → bind-chain → graph and labelling both channels at each boundary
- [x] 7.3 Document the per-sensor health convention: each sensor closure returns an `f64 ∈ [0.0, 1.0]` (1.0 = perfectly healthy); `AggregateLogic::All` aggregates via the product `∏ p_i`, producing a smooth deterioration signal as any sensor degrades. State that `Aggregatable for f64` interprets `All` as a product (this is non-obvious; surface it in the README)
- [x] 7.4 Document the verdict-from-state pattern: the graph's value channel is `V = FlightStateEstimate` end-to-end (per the trait constraint `V_in == V_out`); the `SafetyVerdict` is derived from `final_state.risk` in `main.rs`, not from the value channel. Explain why this is the natural reading of the constraint
- [x] 7.5 Document each field of `FlightState` and `AircraftConfig` with a one-line purpose
- [x] 7.6 Provide the `cargo run -p avionics_examples --example flight_envelope_monitor` invocation and a sample of the expected output (truncated `EffectLog`)
- [x] 7.7 Note that the Kalman step is illustrative (one-iteration scalar update on a diagonal covariance) and link to `magnav` for a more developed filter pattern

## 8. Verification

- [x] 8.1 Run `cargo build -p avionics_examples` from the repository root and confirm zero errors and zero warnings
- [x] 8.2 Run `cargo run -p avionics_examples --example flight_envelope_monitor` and confirm both the "Nominal" and "Failing sensor" sections print as specified, with the program exiting with status zero
- [x] 8.3 Run `cargo fmt -p avionics_examples` (or `make format`) and `cargo clippy -p avionics_examples --all-targets -- -D warnings` (or `make fix`); confirm zero diffs introduced and zero clippy warnings
- [x] 8.4 Only the `avionics_examples` crate was touched (plus its workspace-internal new path dependency on `deep_causality`); the single-crate verification of step 8.3 is sufficient. Do NOT run the full `make build && make test` unless three or more crates were modified
- [x] 8.5 Inspect the generated `EffectLog` from the nominal run and confirm it contains entries from the sensor collection stage, all three bind-chain step labels (`stage2.health_fold`, `stage2.kalman`, `stage2.estimate`), and at least one envelope-graph node, in chronological order
- [x] 8.6 Inspect the generated `EffectLog` from the failing-sensor run and confirm it contains entries from the failing sensor closure but NO entries from the bind-chain steps (`stage2.*`) and NO entries from envelope nodes — proving the short-circuit guarantee
- [x] 8.7 Inspect `final_process.state` from the failing-sensor run and confirm it equals `FlightState::default()` (the state carried at the moment of failure, since no successful step preceded the failing sensor)

## 9. Commit handoff

- [x] 9.1 Prepare a commit message summarising the new example and listing the new and modified files (`examples/avionics_examples/flight_envelope_monitor/{main.rs, model.rs, README.md}`, `examples/avionics_examples/Cargo.toml`, `examples/avionics_examples/README.md`). Per AGENTS.md golden rule #1, do NOT commit; ask the user to perform the `git commit`
