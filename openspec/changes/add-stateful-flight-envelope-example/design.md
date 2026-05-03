## Context

The framework extension `add-stateful-causaloid-evaluation` (archived
2026-05-03) added three parallel traits — `StatefulMonadicCausable`,
`StatefulMonadicCausableCollection`, `StatefulMonadicCausableGraphReasoning` —
that thread `S` (state) and `C` (context) through Causaloid evaluation in
all three causal forms. All three return `PropagatingProcess<O, S, C>` (the
existing public alias for
`CausalEffectPropagationProcess<O, S, C, CausalityError, EffectLog>`). The
existing stateless `MonadicCausable*` traits remain unchanged.

The repository's existing avionics examples (`geometric_tcas`,
`hypersonic_2t`, `magnav`) each demonstrate one Causaloid pattern in isolation.
None of them composes a collection, a `CausalMonad` `bind`-chain, and a
hypergraph end-to-end with non-trivial `State` and `Context`.

The proposed example fills that gap with a compact, runnable avionics
flight-envelope monitor that uses the new stateful evaluation traits and
demonstrates **smooth deterioration trending** via probabilistic sensor health
aggregation rather than a discrete trip threshold.

## Goals / Non-Goals

**Goals:**

- Demonstrate uniform composition: the output of
  `evaluate_collection_stateful` flows into a `CausalMonad` `bind`-chain whose
  output flows into `evaluate_subgraph_from_cause_stateful`, with no glue code
  beyond the `bind` callbacks.
- Carry a real `State` (Kalman covariance + accumulated risk + estimate vector)
  across all three stages.
- Carry a real `Context` (aircraft configuration: mass, MTOW, stall margin,
  service ceiling) read-only across all three stages.
- Show that `EffectLog` aggregates across stages, and that an error in any
  stage short-circuits subsequent stages while preserving the accumulated log
  and the `state` at the moment of failure.
- Demonstrate **trend-style** deterioration monitoring: per-sensor f64 health
  probabilities aggregated via `AggregateLogic::All` produce a continuously-
  evolving joint health signal, in contrast to a discrete threshold trip.
- Stay self-contained: no new external dependencies; only workspace-internal
  path dependencies on `deep_causality` and `deep_causality_core`.
- Match the file layout and `[[example]]` registration idioms of the existing
  `examples/avionics_examples/{geometric_tcas, hypersonic_2t, magnav}`.
- Honour AGENTS.md: no `unsafe`, no macros, no `dyn`, idiomatic zero-cost
  abstractions, surgical diffs.

**Non-Goals:**

- Implementing a certifiable, numerically rigorous Kalman filter. The Kalman
  step is illustrative — a one-iteration scalar update suffices to motivate
  carrying covariance in `State`.
- Modeling a complete avionics flight-envelope domain. Six envelope nodes are
  enough to be a real hypergraph, not exhaustive.
- Modifying any library crate. This change is purely additive in the examples
  tree.
- Introducing async, parallelism, or `Tokio`.
- Adding a new public API on any crate.
- Adding tests in the examples tree (per AGENTS.md, examples are runnable
  demonstrations; library behaviour is tested in the library's own `tests/`
  tree, including the stateful-evaluation tests added by the framework
  extension).

## Decisions

### Decision 1: Use the public alias `PropagatingProcess<T, FlightState, AircraftConfig>` throughout

**Choice**: All three stages produce, consume, and chain values typed as
`PropagatingProcess<T, FlightState, AircraftConfig>` (varying only in `T`,
the value-channel type). Internally `PropagatingProcess<T, S, C>` is exactly
`CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>`; the
public alias is the documented short form.

**Rationale**: Consistency with `deep_causality_core`'s public surface; less
visual noise than the long-form name in `model.rs`. The alias is already
re-exported at the `deep_causality` crate root.

**Alternatives considered**: Define a local crate alias
`FlightProcess<T> = PropagatingProcess<T, FlightState, AircraftConfig>` inside
`model.rs`. Acceptable refinement; the spec leaves this as an
implementation-style choice (the requirement is the underlying type, not the
alias name).

### Decision 2: Three-stage pipeline (collection → bind-chain → graph), not collapsed

**Rationale**: Each shape is the *natural* fit for its stage:

- Stage 1 is N independent observations rolled up to one signal — this is
  what `AggregateLogic` on a Causaloid collection exists for. A bind-chain
  would force artificial sequencing on parallel readings.
- Stage 2 is three pure transforms with hard ordering (health-fold →
  Kalman-update → estimate). A graph would add topology overhead with no
  branching to justify it; bind is the right shape.
- Stage 3 has cross-influence between failure modes (icing raises stall risk;
  low altitude amplifies stall risk into terrain conflict). A hypergraph is
  its native expression.

**Alternatives considered**: collapse to a single graph or a single
bind-chain. Both rejected — they hide the framework's three-shape expressive
range.

### Decision 3: Per-sensor f64 = health probability; aggregate via `AggregateLogic::All`

**Choice**: Each sensor's closure returns an `f64 ∈ [0.0, 1.0]` interpreted
as that sensor's health probability (`1.0` = perfectly healthy, `0.0` =
fully degraded). The collection aggregates via `AggregateLogic::All`, which
the framework's `Aggregatable for f64` impl interprets as the **product**
`∏ p_i` — the joint health probability under independent-sensor
assumption.

**Why `All`**: As any one sensor degrades, the product drops smoothly. This
gives the bind-chain a continuous deterioration signal to fold into
`state.risk`, which is the example's pedagogical centre — showing how the
State channel carries a smoothly-evolving signal that the discrete value
channel cannot.

**The framework f64 caveat (worth surfacing in code comments)**: The framework's
`Aggregatable for f64` interprets values as **probabilities of activation**,
not as health-percent. Two equivalent conventions exist:

- *Health convention* (chosen here): per-sensor f64 = health probability;
  aggregator = `All` (product); display reads forward — `health_pct =
  aggregated * 100`.
- *Anomaly convention* (rejected): per-sensor f64 = anomaly probability;
  aggregator = `Any` (noisy-OR `1 − ∏(1 − q_i)`); display flips —
  `health_pct = (1 − aggregated) * 100`.

Numerically: `aggregated_health_via_All = 1 − noisy_OR_of_anomalies`. Same
information, different framing. The health convention reads forward and the
variable name and number tell the same story.

**Alternatives considered**:

- `AggregateLogic::Some(k)` (the previous draft's choice). **Rejected**:
  the framework's f64 impl returns hard `1.0` or `0.0` for `Some(k)` —
  no gradient, defeats the deterioration-trend goal.
- `AggregateLogic::Any` with the anomaly convention. **Rejected**:
  pedagogically equivalent but requires a display flip; less direct.
- `AggregateLogic::None`. **Rejected**: indistinguishable from `All`
  numerically when sensors are independent; `All` is more idiomatic.

### Decision 4: `State = FlightState { estimate: [f64; 4], covariance: [f64; 4], risk: f64 }`

**Choice**: A 4-element estimate vector (airspeed, altitude, attitude,
vertical speed) with a diagonal covariance and a scalar `risk` accumulator.
Derives `Default + Clone + Debug` (Debug is required by the new
`StatefulMonadicCausable` trait bounds for log formatting).

**Rationale**: The smallest representation that lets the Kalman step do
something observable while keeping the example readable. `risk` is the
load-bearing demonstration of why State exists — it accumulates contributions
from Stage 1 (sensor degradation) AND Stage 3 (envelope nodes), with Stage 2
carrying it forward; the value channel cannot carry this because the value
type changes between stages.

**Alternatives considered**: full 4×4 covariance matrix (rejected — pulls in
matrix ergonomics); risk in the value channel (rejected — risk is
process-level Markovian state, not the per-step value being transformed).

### Decision 5: `Context = AircraftConfig` is `Clone`, read-only

**Rationale**: `Option<Context>` is passed by value into closures during
stateful evaluation. Aircraft configuration (mass, MTOW, stall-margin
multiplier, service ceiling) does not change during a monitor cycle; making
it the `Context` type honours the framework's intent and keeps it out of the
mutable `State`. `AircraftConfig` derives `Clone + Debug` (Debug not strictly
required by trait bounds but kept for ergonomics).

### Decision 6: Stage-3 graph value channel is `V = FlightStateEstimate` (input == output)

**Choice**: `StatefulMonadicCausableGraphReasoning<V, S, C>` is implemented
for `CausaloidGraph<Causaloid<V, V, S, C>>` — the framework constrains the
graph's input value type to equal its output value type. The graph operates
with `V = FlightStateEstimate` end-to-end; each node reads
`FlightStateEstimate` from the value channel and `AircraftConfig` from the
context, then accumulates an envelope-specific risk increment into
`state.risk` and emits the same `FlightStateEstimate` (potentially refined,
but type-preserving).

The final **`SafetyVerdict`** (`Nominal | Caution | Warning | Failure`) is
derived in `main.rs` from the final `state.risk` after all three stages have
run. The graph does NOT transmute the value channel into a `SafetyVerdict`.

**Rationale**: This is the correct way to read the framework constraint —
it forces the example to use the State channel as the cumulative risk
record, which is precisely the pedagogical point. The verdict-from-state
pattern is also more authentic for avionics: a flight-envelope monitor
typically computes a continuous risk score and applies thresholds at the
display layer, not in the reasoning layer.

**Alternatives considered**:

- `V = SafetyVerdict` with each node mutating an enum in the value channel.
  **Rejected**: the value channel becomes an opaque accumulator, the State
  channel goes underused, and the lesson on State degrades to "we just
  carried Kalman covariance".
- Wrap the graph in an outer `bind` that derives the verdict from the
  graph's output. **Rejected**: adds a fourth stage that is purely
  presentation, blurring the three-stage pedagogical line.

### Decision 7: Six envelope nodes, fixed minimal hypergraph

**Choice**: `stall_risk`, `overspeed_risk`, `terrain_proximity`,
`traffic_conflict`, `icing_risk`, `cg_out_of_limits`. Cross-edges include
`icing_risk → stall_risk`, `terrain_proximity → stall_risk`,
`traffic_conflict → overspeed_risk`. Each node is a one-liner closure
in `model.rs`; topology wired in a single helper function.

**Rationale**: Enough nodes to be a real hypergraph, small enough to read
without scrolling.

### Decision 8: Error short-circuit demonstrated by an explicit failure scenario

**Choice**: `main.rs` runs two named scenarios in sequence: a nominal flight
and a failing-sensor scenario. The failing scenario triggers an error in a
sensor closure (returning `Some(CausalityError(...))` on the per-sensor
process), which the collection's stateful aggregation propagates up — the
bind-chain and the graph then do not execute. Both scenarios print the
final verdict (or error), the final `FlightState` (preserved at moment of
failure for the failing case), and the full `EffectLog`.

**Rationale**: The error short-circuit guarantee — that state at moment of
failure is preserved on the returned process — is invisible if every run
succeeds. The two-scenario format makes it observable. The framework
extension's tests already prove the guarantee at the unit level; this
example demonstrates it at the integration level.

### Decision 9: Layout matches sibling examples; `[[example]]` not `[[bin]]`

**Choice**:
`examples/avionics_examples/flight_envelope_monitor/{main.rs, model.rs, README.md}`,
registered as
`[[example]] name = "flight_envelope_monitor", path = "flight_envelope_monitor/main.rs"`
in `examples/avionics_examples/Cargo.toml`. Invoked via
`cargo run -p avionics_examples --example flight_envelope_monitor`.

**Rationale**: Strict consistency with the existing `magnav`,
`geometric_tcas`, and `hypersonic_2t` registrations in
[examples/avionics_examples/Cargo.toml](examples/avionics_examples/Cargo.toml).
The earlier draft of this spec used `[[bin]]` and `--bin`, which was wrong
— flagged and corrected during the feasibility assessment.

### Decision 10: No Bazel target

**Choice**: No `BUILD.bazel` is added in `examples/avionics_examples/`. The
existing avionics-examples tree has no Bazel files; examples in this
repository are Cargo-only.

**Rationale**: `find examples -name BUILD.bazel` returns nothing. The
earlier draft proposed adding a `rust_binary` target — incorrect. The
repository's Bazel setup covers library crates (`deep_causality_*` and
`ultragraph`) plus `thirdparty/`, not examples.

### Decision 11: No tests in the example crate

**Rationale**: AGENTS.md treats the examples tree as runnable
demonstrations, not test fixtures. The framework extension already provides
unit tests for `evaluate_stateful`, `evaluate_collection_stateful`, and
`evaluate_subgraph_from_cause_stateful`, including state-preservation,
log aggregation, error short-circuit, and `RelayTo`. Adding tests here
would duplicate that coverage and bloat the example.

### Decision 12: Add `deep_causality` as a workspace path dependency on `avionics_examples`

**Choice**: Modify `examples/avionics_examples/Cargo.toml` to add
`deep_causality = { path = "../../deep_causality" }` to `[dependencies]`.

**Rationale**: The current manifest depends on `deep_causality_core`,
`deep_causality_tensor`, and `deep_causality_multivector` only —
`deep_causality` is not yet a dependency of `avionics_examples`. The new
example consumes `Causaloid`, `CausaloidGraph`, the `StatefulMonadicCausable*`
traits, `AggregateLogic`, and a number of other re-exports that live only in
`deep_causality`. This is a workspace-internal path dependency; no
crates.io entries are added.

## Risks / Trade-offs

- **Risk**: A reader unfamiliar with probabilistic aggregation may misread
  the f64 "health" channel as percent-health rather than probability.
  → **Mitigation**: README opens with one paragraph spelling out the
  convention (per-sensor f64 = health probability ∈ [0.0, 1.0]; aggregator
  `All` = product = joint health probability). Code comments on each sensor
  closure name what they return.

- **Risk**: The Kalman step is too simplified and a reader may infer the
  framework only supports toy filtering. → **Mitigation**: README explicitly
  states the Kalman step is illustrative and points at `magnav` for a more
  developed filter pattern.

- **Risk**: Six envelope nodes still feels like a lot for an example.
  → **Mitigation**: Each node is a one-liner closure; the hypergraph wiring
  is shown once in a single helper function in `model.rs`.

- **Risk**: Carrying both `State` and `Context` may overwhelm a reader new
  to the framework. → **Mitigation**: README opens with a comparison to the
  stateless `PropagatingEffect<T>` form and labels each field of
  `FlightState` and `AircraftConfig` with its purpose. Stage-1 sensor
  closures use the stateless `Causaloid::new` form (per-sensor closures
  don't need state); the stateful path enters at the collection-level
  evaluation, which is a gentler introduction.

- **Risk**: The verdict-from-state pattern (Decision 6) may surprise a
  reader expecting the value channel to carry the verdict. → **Mitigation**:
  README's pipeline diagram explicitly shows the value channel transitions
  (`SensorReading → f64 → FlightStateEstimate → FlightStateEstimate`) and
  the State channel transitions (`risk: 0 → contributions → final`), and
  states that the verdict is read from State, not from the value channel.

- **Trade-off**: `state.risk` is only meaningfully populated by the bind
  chain and the envelope graph; per-sensor causaloids don't mutate State.
  This is by design — Stage 1's contribution to `risk` is folded by the
  *bind-chain's first step*, not by the per-sensor closures, because the
  per-sensor closures use stateless `Causaloid::new` and their f64 outputs
  feed the aggregator. Spec scenarios reflect this division of labour.

- **Trade-off**: Five sensors and six envelope nodes inflate the line count
  of `model.rs`. Acceptable — the example is meant to be a reference, not a
  one-screen demo.
