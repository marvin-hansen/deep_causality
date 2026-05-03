## 1. Scaffolding

- [ ] 1.1 Create directory `examples/avionics_examples/flight_envelope_monitor/`
- [ ] 1.2 Create empty `main.rs`, `model.rs`, and `README.md` files in that directory
- [ ] 1.3 Add a `[[bin]]` target named `flight_envelope_monitor` to `examples/avionics_examples/Cargo.toml` pointing at `flight_envelope_monitor/main.rs`, mirroring the existing entries for `geometric_tcas`, `hypersonic_2t`, and `magnav`
- [ ] 1.4 Add a `rust_binary` target for `flight_envelope_monitor` to `examples/avionics_examples/BUILD.bazel`, mirroring the existing avionics binary targets
- [ ] 1.5 Append a one-line entry for `flight_envelope_monitor` to `examples/avionics_examples/README.md`
- [ ] 1.6 Verify `cargo build -p avionics_examples` succeeds with the empty files in place

## 2. Domain types in `model.rs`

- [ ] 2.1 Define `FlightState { estimate: [f64; 4], covariance: [f64; 4], risk: f64 }` deriving `Default`, `Clone`, and `Debug`
- [ ] 2.2 Define `AircraftConfig { mass_kg: f64, mtow_kg: f64, stall_margin: f64, service_ceiling_m: f64 }` deriving `Clone` and `Debug`
- [ ] 2.3 Define `SensorReading { airspeed_kn: f64, altitude_ft: f64, attitude_deg: f64, vertical_speed_fpm: f64, fuel_flow_pph: f64 }` deriving `Clone` and `Debug`
- [ ] 2.4 Define `SensorHealth`, `FlightStateEstimate`, and `SafetyVerdict` enums or structs as the value-channel payload between stages, all deriving `Default`, `Clone`, and `Debug`
- [ ] 2.5 Define a local type alias `FlightProcess<T> = CausalEffectPropagationProcess<T, FlightState, AircraftConfig, CausalityError, EffectLog>`

## 3. Stage 1 — sensor causaloid collection

- [ ] 3.1 Implement five singleton `Causaloid` constructors in `model.rs`, one per sensor field, each using `Causaloid::new` (or `new_with_context` if the threshold depends on `AircraftConfig`)
- [ ] 3.2 Implement a builder function `build_sensor_collection(config: AircraftConfig) -> Causaloid<...>` that wraps the five sensors via `Causaloid::from_causal_collection_with_context`, using `AggregateLogic` with a numeric threshold of 2 (i.e. trip on 2-of-5 anomalous sensors)
- [ ] 3.3 Verify the resulting `Causaloid` has `is_singleton() == false` and that its evaluation with a nominal `SensorReading` produces a non-error `FlightProcess<SensorHealth>` whose `EffectLog` contains aggregation entries

## 4. Stage 2 — `CausalMonad` bind-chain

- [ ] 4.1 Implement `validate_units(value, state, context) -> FlightProcess<FlightStateEstimate>` as the first bind step; emit an error if the airspeed lies outside `[0, 2 * stall_margin * MTOW-derived ceiling]` or similar guard, otherwise pass through with a log entry
- [ ] 4.2 Implement `kalman_step(value, state, context) -> FlightProcess<FlightStateEstimate>` as the second bind step; update `state.covariance` via a one-iteration scalar Kalman update and write a log entry that names the step
- [ ] 4.3 Implement `estimate_state(value, state, context) -> FlightProcess<FlightStateEstimate>` as the third bind step; write the estimated state into `state.estimate` and accumulate a sensor-derived risk increment into `state.risk`
- [ ] 4.4 Compose the three steps via `bind_or_error` (or `bind` where appropriate) on the `FlightProcess<SensorHealth>` returned by Stage 1; use `CausalEffectPropagationProcess::with_state` once at the boundary if needed to lift a stateless intermediate
- [ ] 4.5 Verify that running the chain on a nominal `SensorHealth` produces a non-error `FlightProcess<FlightStateEstimate>` whose `state.covariance` differs from the input covariance and whose `EffectLog` contains entries for all three steps

## 5. Stage 3 — envelope causaloid hypergraph

- [ ] 5.1 Implement six singleton context-aware causaloids — `stall_risk`, `overspeed_risk`, `terrain_proximity`, `traffic_conflict`, `icing_risk`, `cg_out_of_limits` — each via `Causaloid::new_with_context`, reading the relevant fields from `FlightStateEstimate` and `AircraftConfig`
- [ ] 5.2 Build a `CausaloidGraph` that wires the six nodes with at least one cross-edge per node (e.g. `icing_risk → stall_risk`, `terrain_proximity ↔ stall_risk`, `traffic_conflict → overspeed_risk`); document the topology in a comment
- [ ] 5.3 Wrap the graph via `Causaloid::from_causal_graph_with_context` with the supplied `AircraftConfig`
- [ ] 5.4 Implement an aggregation function that maps the graph's evaluation output into a `SafetyVerdict` (e.g. `Nominal | Caution | Warning | Failure`) and adds an envelope-derived risk increment into `state.risk`
- [ ] 5.5 Verify that evaluating the wrapped graph on a nominal `FlightProcess<FlightStateEstimate>` produces a non-error `FlightProcess<SafetyVerdict>` whose log includes envelope-stage entries

## 6. Demonstration in `main.rs`

- [ ] 6.1 Construct a nominal `AircraftConfig` and a nominal `SensorReading`
- [ ] 6.2 Compose the three stages: build the sensor collection, evaluate it on the reading, lift to a stateful process via `with_state` if needed, run the bind-chain, then evaluate the envelope graph
- [ ] 6.3 Print a section labeled "Nominal" containing the final `SafetyVerdict`, the final `FlightState`, and the full `EffectLog`
- [ ] 6.4 Construct a failing `SensorReading` (e.g. airspeed sensor returns `EffectValue::None` or a value outside guarded bounds)
- [ ] 6.5 Run the same composition on the failing reading
- [ ] 6.6 Print a section labeled "Failing sensor" containing the `CausalityError` from the final process, the `FlightState` at the moment of failure, and the partial `EffectLog`
- [ ] 6.7 Verify the program exits with status zero in both scenarios (the failing scenario surfaces its error through stdout, not via `std::process::exit`)

## 7. README

- [ ] 7.1 In `examples/avionics_examples/flight_envelope_monitor/README.md`, write an opening paragraph that contrasts `CausalEffectPropagationProcess<_, FlightState, AircraftConfig, _, _>` with the simplified `PropagatingEffect<T>` alias and states why this example needs the full form
- [ ] 7.2 Document the three-stage pipeline with a small ASCII diagram showing collection → bind-chain → hypergraph and labeling the value type at each boundary
- [ ] 7.3 Document each field of `FlightState` and `AircraftConfig` with a one-line purpose
- [ ] 7.4 Provide the `cargo run -p avionics_examples --bin flight_envelope_monitor` invocation and a sample of the expected output (truncated `EffectLog`)
- [ ] 7.5 Note that the Kalman step is illustrative and link to a more developed filter pattern in a sibling example

## 8. Verification

- [ ] 8.1 Run `cargo build -p avionics_examples` from the repository root and confirm zero errors and zero warnings
- [ ] 8.2 Run `cargo run -p avionics_examples --bin flight_envelope_monitor` and confirm both the nominal and failing-sensor sections print as specified
- [ ] 8.3 Run `make format` and `make fix` from the repository root and confirm zero diffs introduced and zero clippy warnings
- [ ] 8.4 If three or more crates were touched in the final state of the change, also run `make build` and `make test`; otherwise build and test the affected crate only
- [ ] 8.5 Inspect the generated `EffectLog` from the nominal run and confirm it contains entries from all three stages in chronological order
- [ ] 8.6 Inspect the generated `EffectLog` from the failing-sensor run and confirm it contains entries from the stages executed up to and including the failing stage, and no entries from later stages

## 9. Commit handoff

- [ ] 9.1 Prepare a commit message summarizing the change and listing the new and modified files; do not commit. Per AGENTS.md golden rule #1, ask the user to perform the git commit.
