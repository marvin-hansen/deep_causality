## Why

The `deep_causality` crate carries non-trivial `STATE` and `CTX` type parameters
on every `Causaloid<I, O, STATE, CTX>` and stores closures of type
`ContextualCausalFn<I, O, S, C>` which **already** return the stateful alias
`PropagatingProcess<O, S, C> = CausalEffectPropagationProcess<O, S, C, CausalityError, EffectLog>`.
However, the public evaluation path at the trait-method layer collapses this back to
the stateless `PropagatingEffect<O> = CausalEffectPropagationProcess<O, (), (), CausalityError, EffectLog>`:

- `MonadicCausable::evaluate` returns `PropagatingEffect<O>`
  ([traits/causable/mod.rs:26](deep_causality/src/traits/causable/mod.rs#L26)).
- `MonadicCausableCollection::evaluate_collection` returns `PropagatingEffect<O>`
  ([traits/causable_collection/.../monadic_collection.rs:45](deep_causality/src/traits/causable_collection/collection_reasoning/monadic_collection.rs#L45)).
- `MonadicCausableGraphReasoning::evaluate_single_cause` /
  `evaluate_subgraph_from_cause` return `PropagatingEffect<V>`
  ([traits/causable_graph/graph_reasoning/mod.rs:40](deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs#L40)).
- `execute_causal_logic` invokes the stateful closure with a freshly defaulted
  `PS::default()` and discards the returned state during conversion to
  `PropagatingEffect<O>`
  ([types/causal_types/causaloid/causable_utils.rs:41-56](deep_causality/src/types/causal_types/causaloid/causable_utils.rs#L41-L56)).

The consequence is that consumers cannot thread Markovian state (e.g. a Kalman
covariance) or read-only context (e.g. an aircraft configuration) through a
Causaloid evaluation — even though the closure layer is fully equipped to
support it. Stateful causal reasoning is currently expressible only inside a
hand-rolled `CausalMonad` `bind`-chain, and uniform composition between a
`Causaloid` and a `CausalMonad` is therefore restricted to the stateless form.

This change closes that gap by adding a parallel stateful evaluation API at the
trait-method layer for the three causal forms (singleton, collection, graph).
The change is purely additive: existing `MonadicCausable*` traits and methods
are unchanged.

## What Changes

- Add a new public trait `StatefulMonadicCausable<I, O, S, C>` exposing
  `evaluate_stateful(&self, &PropagatingProcess<I, S, C>) -> PropagatingProcess<O, S, C>`
  that threads `S` and `C` from the incoming process through the causaloid's
  context-aware closure and returns the resulting stateful process without
  collapsing it.
- Add a new public trait `StatefulMonadicCausableCollection<I, O, S, C, T>`
  exposing `evaluate_collection_stateful(...)` that aggregates child causaloids
  while preserving the `S` and `C` carried by the incoming process.
- Add a new public trait `StatefulMonadicCausableGraphReasoning<V, S, C>`
  exposing `evaluate_single_cause_stateful(...)`,
  `evaluate_subgraph_from_cause_stateful(...)`, and
  `evaluate_shortest_path_between_causes_stateful(...)` paralleling the
  existing stateless graph reasoning methods.
- Implement all three new traits for `Causaloid<I, O, S, C>` and the existing
  graph types so that `Causaloid` of type `Singleton`, `Collection`, and
  `Graph` are uniformly evaluable in stateful form.
- Refactor `execute_causal_logic` to additionally expose a stateful sibling
  `execute_causal_logic_stateful` that does **not** default `PS` and does
  **not** discard the resulting state. The original function is left
  unchanged for backward compatibility with the existing stateless evaluate.
- Add a public type alias
  `StatefulContextualCausalFn<I, O, S, C> = fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>`
  in `deep_causality/src/alias/alias_function.rs`, alongside the existing
  `ContextualCausalFn`. Structurally the two aliases are identical (the
  existing `ContextualCausalFn` already has the stateful return shape); the
  new alias is a clearly-named ergonomic marker at the closure-author site
  for closures intended to be evaluated via the new stateful traits.
- **No new `Causaloid` constructors are introduced.** Stateful evaluation
  is determined entirely by which trait method the caller invokes
  (`evaluate` vs `evaluate_stateful`, `evaluate_collection` vs
  `evaluate_collection_stateful`, and the graph reasoning equivalents); no
  existing `Causaloid` constructor makes a state-threading decision.
  Adding a "stateful" naming variant on any constructor would imply a
  guarantee the constructor cannot enforce (the same `Causaloid` can be
  evaluated either statelessly or statefully). The rustdoc on each new
  trait method directs the reader to the existing `_with_context`
  constructors and recommends declaring closures via the
  `StatefulContextualCausalFn` alias for clarity at the closure-author
  site.
- Re-export the new traits from `deep_causality::lib` next to the existing
  `MonadicCausable*` re-exports. Re-export `StatefulContextualCausalFn` as
  well, paralleling the existing `ContextualCausalFn` re-export.
- Add comprehensive unit tests covering each new trait method for each causal
  form, including state evolution across multiple steps, error
  short-circuiting that preserves accumulated state, and log aggregation.
- Document in the rustdoc of each new trait method:
  - The relationship to the stateless counterpart (one is not a replacement
    for the other).
  - The semantics of state threading on error (state at moment of failure is
    preserved on the returned process).
  - When to choose the stateful variant over the stateless one.

**No breaking changes.** The existing `MonadicCausable`,
`MonadicCausableCollection`, and `MonadicCausableGraphReasoning` traits, their
methods, and their existing implementations are not modified. All callers of
the stateless API continue to compile and behave identically.

## Capabilities

### New Capabilities

- `stateful-causaloid-evaluation`: A public, additive evaluation surface on
  `Causaloid` (singleton, collection, and graph forms) that threads
  user-defined `State` and `Context` through evaluation without collapsing the
  result to the stateless `PropagatingEffect`. Covers the three new traits,
  their `Causaloid` implementations, the stateful `execute_causal_logic`
  helper, the rustdoc semantics, and the test surface.

### Modified Capabilities

<!-- None. There are no existing OpenSpec capability specs in
openspec/specs/. The new traits are strictly additive and do not alter the
behavioral contracts of the existing stateless `MonadicCausable*` traits. -->

## Impact

- **New code**:
  - `deep_causality/src/traits/causable/stateful.rs` — new
    `StatefulMonadicCausable<I, O, S, C>` trait.
  - `deep_causality/src/traits/causable_collection/collection_reasoning/stateful_monadic_collection.rs`
    — new `StatefulMonadicCausableCollection<I, O, S, C, T>` trait.
  - `deep_causality/src/traits/causable_graph/graph_reasoning/stateful.rs` —
    new `StatefulMonadicCausableGraphReasoning<V, S, C>` trait.
  - `deep_causality/src/types/causal_types/causaloid/causable_stateful.rs` —
    `Causaloid` implementations of the three new traits.
  - `deep_causality/src/types/causal_types/causaloid/causable_utils.rs` —
    extended with `execute_causal_logic_stateful` (one new function in the
    same file; the existing `execute_causal_logic` is untouched).
  - `deep_causality/tests/traits/causable_stateful_tests.rs`,
    `deep_causality/tests/traits/causable_collection_stateful_tests.rs`,
    `deep_causality/tests/traits/causable_graph_stateful_tests.rs` — new test
    files following the repository's mirror-the-source-tree convention.
- **Modified code**:
  - `deep_causality/src/alias/alias_function.rs` — add the
    `StatefulContextualCausalFn<I, O, S, C>` type alias adjacent to the
    existing `ContextualCausalFn` alias.
  - `deep_causality/src/types/causal_types/causaloid/mod.rs` — register the
    new submodule `causable_stateful`. No existing constructors are
    modified; no new constructors are added.
  - `deep_causality/src/traits/causable/mod.rs` — register the new submodule
    `stateful` (no change to the existing `MonadicCausable` trait).
  - `deep_causality/src/traits/causable_collection/collection_reasoning/mod.rs`
    — register the new submodule `stateful_monadic_collection`.
  - `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs` —
    register the new submodule `stateful` (no change to the existing trait).
  - `deep_causality/src/lib.rs` — re-export the three new traits and the
    `StatefulContextualCausalFn` alias at crate root.
  - `deep_causality/tests/mod.rs` (and the `traits/` test mod file) —
    register the new test files per the repository's strict test-mod
    convention.
- **APIs touched**: All additive. No existing public symbol is renamed,
  removed, or has its signature changed.
- **Dependencies**: No new external crates. The change uses only
  `deep_causality_core` (for `PropagatingProcess`, `EffectValue`,
  `CausalityError`, `EffectLog`) and `deep_causality_haft` (for `LogAppend`
  and `LogAddEntry`), both of which are already workspace dependencies of
  `deep_causality`.
- **Performance**: The stateful path adds one extra clone of `S` and one
  optional clone of `C` per evaluation step compared to the stateless path
  (which uses `PS::default()` and `Option<C>::None`). Acceptable for
  ergonomic state threading and consistent with the existing log-cloning cost
  of the stateless path.
- **Documentation**: Each new trait carries module-level rustdoc explaining
  its relationship to the stateless counterpart and recommending when to use
  it. No changes to the project README or AGENTS.md.
- **Risk**: Low for the framework extension itself (additive, behind new
  trait names, fully tested). The follow-up flight-envelope-monitor example
  change (`add-stateful-flight-envelope-example`) becomes implementable once
  this lands.
