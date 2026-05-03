## Context

The DeepCausality framework defines `CausalEffectPropagationProcess<Value, State,
Context, Error, Log>` as the single carrier type produced by both the
`MonadicCausable::evaluate` method on a `Causaloid` and by `MonadEffect5::pure /
bind` on a `CausalMonad`. The simplified alias `PropagatingEffect<T>` fixes
`State = ()` and `Context = ()` and is sufficient for stateless, pure-functional
chains. The full process type is intended for chains that need to carry
Markovian state (e.g. a Kalman covariance) and read-only configuration (e.g.
aircraft limits) across stages.

The repository's existing avionics examples (`geometric_tcas`, `hypersonic_2t`,
`magnav`) each demonstrate one Causaloid pattern in isolation. None of them:

- Composes a `Causaloid` collection, a `CausalMonad` bind-chain, and a
  `Causaloid` hypergraph end-to-end.
- Uses `CausalEffectPropagationProcess` with non-trivial `State` and `Context`.
- Shows the audit-log accumulation and error short-circuit behavior across
  heterogeneous causal stages.

The proposed example fills that gap with a compact, runnable avionics flight-
envelope monitor.

## Goals / Non-Goals

**Goals:**

- Demonstrate uniform composition: `Causaloid::evaluate` output flows directly
  into `CausalMonad::bind`, whose output flows directly into another
  `Causaloid::evaluate`, with no glue code beyond the `bind` callback.
- Carry a real `State` (Kalman covariance + accumulated risk score) across all
  three stages.
- Carry a real `Context` (aircraft configuration: mass, MTOW, stall margin,
  service ceiling) read-only across all three stages.
- Show that `EffectLog` aggregates across stages, and that an error in any
  stage short-circuits subsequent stages while preserving the accumulated log.
- Stay self-contained: no new external dependencies; uses only `deep_causality`
  and `deep_causality_core` from the workspace.
- Match the file layout and idioms of the existing
  `examples/avionics_examples/{geometric_tcas, hypersonic_2t, magnav}`
  examples.

**Non-Goals:**

- Implementing a certifiable, numerically rigorous Kalman filter. The Kalman
  step is illustrative — a single-iteration scalar update suffices to motivate
  carrying covariance in `State`.
- Modeling a complete avionics flight-envelope domain. The hypergraph contains
  enough nodes (stall, overspeed, terrain proximity, traffic conflict, icing,
  CG-out-of-limits) to be non-trivial but is not an exhaustive flight model.
- Modifying any library crate. This change is purely additive in the examples
  tree.
- Introducing async, parallelism, or `Tokio`. The example is synchronous.
- Adding or modifying a public API on `deep_causality`, `deep_causality_core`,
  or any other crate.

## Decisions

### Decision 1: Use `CausalEffectPropagationProcess<_, FlightState, AircraftConfig, CausalityError, EffectLog>` directly, not `PropagatingEffect<T>`

**Rationale**: The example exists specifically to demonstrate the stateful
form. Using the `PropagatingEffect<T>` alias (which forces `State = ()` and
`Context = ()`) would defeat the purpose. The constructor
`CausalEffectPropagationProcess::with_state` lifts a stateless effect into the
stateful process and is the documented entry point for this scenario.

**Alternatives considered**:

- `PropagatingEffect<T>` — rejected: forces unit state and context, blocking
  the Kalman covariance demonstration.
- A custom alias `FlightProcess<T>` — accepted as a local type alias inside
  `model.rs` for readability, but the underlying type remains
  `CausalEffectPropagationProcess<T, FlightState, AircraftConfig, CausalityError, EffectLog>`.

### Decision 2: Three-stage pipeline (collection → monad → graph), not collapsed

**Rationale**: Each shape is the *natural* fit for its stage:

- Stage 1 is N independent observations rolled up to one verdict — this is
  what `AggregateLogic` on a `Causaloid::from_causal_collection` exists for.
  A bind-chain would force artificial sequencing on parallel readings.
- Stage 2 is four pure transforms with hard ordering (validate → unit-convert
  → Kalman update → estimate). A graph would add topology overhead with no
  branching to justify it; bind is the right shape.
- Stage 3 has cross-influence between failure modes (e.g. icing raises stall
  risk; low altitude amplifies stall risk into terrain conflict). This is a
  hypergraph and `Causaloid::from_causal_graph_with_context` is its native
  expression.

**Alternatives considered**:

- A single deeply-nested `Causaloid` graph for everything — rejected: the
  monad-shaped middle stage becomes opaque; the example loses pedagogical
  clarity on why both abstractions exist.
- A single `bind` chain for everything — rejected: parallel sensor aggregation
  and the envelope hypergraph become hand-rolled folds, hiding the framework's
  expressive power.

### Decision 3: Sensor aggregation uses `AggregateLogic` with a count threshold

**Rationale**: A safety monitor should trip on N or more anomalous sensors,
not on any single sensor (false-positive control) and not on all (single
silent-failure tolerance). `AggregateLogic` with a numeric threshold value
(e.g. 2-of-5) expresses this directly.

**Alternatives considered**:

- `AggregateLogic::Any` — rejected as too eager for a safety verdict in this
  illustrative context.
- `AggregateLogic::All` — rejected: a single failed sensor would mask the
  system from tripping.

### Decision 4: `State = FlightState { estimate: [f64; 4], covariance: [f64; 4], risk: f64 }`

**Rationale**: A 4-element state vector (airspeed, altitude, attitude, vertical
speed) with a diagonal covariance is the smallest representation that lets the
Kalman step do something observable while keeping the example readable.
`risk` accumulates a scalar safety penalty so downstream stages can reason
about cumulative risk without re-deriving it from the value channel. `Clone`
and `Default` are already required by the trait bounds on
`CausalEffectPropagationProcess`.

**Alternatives considered**:

- Full 4×4 covariance matrix — rejected: pulls in matrix ergonomics that
  obscure the framework demonstration.
- Storing risk in the value channel — rejected: risk is process-level
  Markovian state, not the per-step value being transformed.

### Decision 5: `Context = AircraftConfig` is `Clone` and read-only

**Rationale**: The framework's `bind` signature passes `Option<Context>` by
value into the callback. Aircraft configuration (mass, MTOW, stall-margin
multiplier, service ceiling) does not change during a single monitor cycle;
making it the `Context` type both honors the framework's intent and keeps it
out of the mutable `State`.

### Decision 6: Hypergraph topology is fixed and minimal

**Rationale**: Six envelope nodes (stall, overspeed, terrain proximity,
traffic conflict, icing, CG-out-of-limits) with documented cross-edges. Enough
to be a real hypergraph, small enough to read in `model.rs` without scrolling.

### Decision 7: Error short-circuit demonstrated by an explicit failure scenario in `main.rs`

**Rationale**: The pedagogical point — that an error in any stage skips later
stages while preserving accumulated logs — is invisible if every run of the
example succeeds. `main.rs` will run two scenarios: a nominal flight and a
sensor-failure scenario, printing the resulting verdict and the full
`EffectLog` for each.

### Decision 8: Layout matches sibling examples

`examples/avionics_examples/flight_envelope_monitor/{main.rs, model.rs, README.md}`
plus a binary target entry in
`examples/avionics_examples/Cargo.toml` and a `rust_binary` target in
`examples/avionics_examples/BUILD.bazel`. No deviation from the established
convention.

### Decision 9: No tests in the example crate beyond what `cargo build -p avionics_examples` and `cargo run` verify

**Rationale**: AGENTS.md treats the examples tree as runnable demonstrations,
not as test fixtures. Library behavior is tested in the library crate's own
`tests/` tree. Adding tests here would duplicate that coverage and bloat the
example.

## Risks / Trade-offs

- **Risk**: The Kalman step is too simplified and a reader may infer the
  framework only supports toy filtering. → **Mitigation**: The `README.md`
  explicitly states the Kalman step is illustrative and points to a real
  implementation (e.g. `magnav` for a more developed filter pattern).
- **Risk**: Six envelope nodes still feels like a lot for an example.
  → **Mitigation**: Each node is a one-liner closure in `model.rs`. The
  hypergraph wiring is shown once in a single helper function.
- **Risk**: Carrying both `State` and `Context` may overwhelm a reader new to
  the framework. → **Mitigation**: The `README.md` opens with a one-paragraph
  comparison to the stateless `PropagatingEffect<T>` form and labels each
  field of `FlightState` and `AircraftConfig` with its purpose.
- **Risk**: The example duplicates logic from `geometric_tcas` /
  `hypersonic_2t` / `magnav`. → **Mitigation**: The example is deliberately
  scoped to composition-of-three-stages; it does not re-implement TCAS,
  hypersonic guidance, or magnetic navigation. Naming and helpers are kept
  distinct.
- **Trade-off**: Putting `risk` in `State` rather than in the value channel
  means downstream stages must read it from `state` in the `bind` callback.
  This is the intended demonstration of why `State` exists, but it does
  require the reader to look at both arguments of the `bind` closure.
