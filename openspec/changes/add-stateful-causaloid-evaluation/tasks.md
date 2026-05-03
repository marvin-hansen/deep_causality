## 1. Pre-implementation reading

- [x] 1.1 Re-read `proposal.md`, `design.md`, and `specs/stateful-causaloid-evaluation/spec.md` for this change before writing any code
- [x] 1.2 Re-read `AGENTS.md` to confirm conventions (one type / one module, tests mirror source tree, no `unsafe`, no macros in lib, idiomatic zero-cost abstractions, no new external crates)
- [x] 1.3 Re-read `deep_causality/src/types/causal_types/causaloid/causable_utils.rs` and `deep_causality/src/types/causal_types/causaloid/causable.rs` to anchor on existing implementation patterns
- [x] 1.4 Re-read `deep_causality_core/src/types/propagating_process/mod.rs` and `deep_causality_core/src/types/causal_effect_propagation_process/mod.rs` to confirm the public API of `PropagatingProcess` and the existing `bind` semantics on the underlying process type

## 2. Stateful closure type alias

- [x] 2.1 In `deep_causality/src/alias/alias_function.rs`, add a new public type alias `pub type StatefulContextualCausalFn<I, O, S, C> = fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;` adjacent to the existing `ContextualCausalFn`; do not modify `ContextualCausalFn`
- [x] 2.2 Add rustdoc on `StatefulContextualCausalFn` explaining the relationship to `ContextualCausalFn` (same `fn` pointer shape, parallel name, recommended for closures intended to be evaluated via `StatefulMonadicCausable::evaluate_stateful` and similar stateful trait methods); the rustdoc SHALL also state that the existing `Causaloid::new_with_context` constructor accepts this alias and is the recommended path for authoring a Causaloid intended for stateful evaluation — no new constructor exists or is needed
- [x] 2.3 In `deep_causality/src/lib.rs`, re-export the new alias: `pub use crate::alias::alias_function::StatefulContextualCausalFn;` adjacent to the existing `ContextualCausalFn` re-export (if there is no existing re-export of `ContextualCausalFn` at crate root, then verify the new alias surfaces via the existing `pub use crate::alias::*;` glob)
- [x] 2.4 Confirm by inspection that `deep_causality/src/types/causal_types/causaloid/mod.rs` has zero added `pub fn` items in any `impl` block on `Causaloid` after this change (i.e. no new constructors are introduced)

## 3. Stateful execute_causal_logic helper

- [x] 3.1 Add a new function `execute_causal_logic_stateful<I, O, S, C>(input: I, state: S, context: Option<C>, causaloid: &Causaloid<I, O, S, C>) -> PropagatingProcess<O, S, C>` to `deep_causality/src/types/causal_types/causaloid/causable_utils.rs` (same file, additive, do not touch `execute_causal_logic`)
- [x] 3.2 In the new function, when `causaloid.context_causal_fn` is set, invoke it with the supplied `state` and `context` (do not call `S::default()`) and return the resulting `PropagatingProcess<O, S, C>` unchanged (no conversion, no log loss)
- [x] 3.3 In the new function, when only `causaloid.causal_fn` is set, invoke the stateless closure on the value and lift the resulting `PropagatingEffect<O>` into a `PropagatingProcess<O, S, C>` whose `state` and `context` fields are the pass-through `state` and `context` passed in by the caller and whose `value`, `error`, `logs` come from the stateless effect
- [x] 3.4 In the new function, when neither closure is set, return a `PropagatingProcess::from_error(...)` with a precise error message identifying the causaloid id, preserving the supplied `state` and `context` on the returned process
- [x] 3.5 Export the new helper as `pub(super)` (same visibility as `execute_causal_logic`)

## 4. StatefulMonadicCausable trait

- [x] 4.1 Create file `deep_causality/src/traits/causable/stateful.rs`
- [x] 4.2 Define `pub trait StatefulMonadicCausable<I, O, S, C>` with one method `fn evaluate_stateful(&self, incoming: &PropagatingProcess<I, S, C>) -> PropagatingProcess<O, S, C>;`
- [x] 4.3 Add module-level rustdoc explaining the relationship to `MonadicCausable<I, O>` (when to use which, that they coexist, that this one preserves `S` and `C`)
- [x] 4.4 Add per-method rustdoc covering: state-threading semantics, log aggregation, error short-circuit semantics (state at moment of failure is preserved), and the pass-through behavior when the underlying causaloid uses the stateless `causal_fn`
- [x] 4.5 In the rustdoc for `evaluate_stateful`, explicitly direct the reader to the existing `Causaloid::new_with_context` constructor as the way to author a Causaloid intended for stateful evaluation, and to the `StatefulContextualCausalFn` type alias as the recommended type for the closure argument; state that statefulness is a property of the **call** (which trait method is invoked), not of the **constructor**
- [x] 4.6 Register the new submodule in `deep_causality/src/traits/causable/mod.rs` (add `pub mod stateful;` next to the existing module declarations); do not modify the existing `MonadicCausable` trait

## 5. Causaloid impl of StatefulMonadicCausable

- [x] 5.1 Create file `deep_causality/src/types/causal_types/causaloid/causable_stateful.rs`
- [x] 5.2 Add `use` lines for the new trait, `Causaloid`, `CausaloidType`, `CausalityError`, `PropagatingProcess`, `EffectValue`, `causable_utils::execute_causal_logic_stateful`, and any helper macros from `deep_causality_haft` already in use by `causable.rs` (notably `LogAddEntry`)
- [x] 5.3 Implement `StatefulMonadicCausable<I, O, S, C> for Causaloid<I, O, S, C>` with the same trait bounds as the existing `MonadicCausable` impl (added `Debug` to `PS` to enable log formatting)
- [x] 5.4 Inside `evaluate_stateful`, branch on `self.causal_type` (chain log_input_stateful → execute_causal_logic_stateful → log_output_stateful for Singleton; precise error process for Collection / Graph)
- [x] 5.5 Add stateful versions of `log_input` and `log_output` in `causable_utils.rs` reusing the existing log message format strings
- [x] 5.6 Register the new submodule in `deep_causality/src/types/causal_types/causaloid/mod.rs` next to `mod causable;` and `mod causable_utils;`

## 6. StatefulMonadicCausableCollection trait and impl

- [x] 6.1 Create file `deep_causality/src/traits/causable_collection/collection_reasoning/stateful_monadic_collection.rs`
- [x] 6.2 Define `pub trait StatefulMonadicCausableCollection<I, O, S, C, T>: CausableCollectionAccessor<I, O, T>` with method `fn evaluate_collection_stateful(&self, incoming: &PropagatingProcess<I, S, C>, logic: &AggregateLogic, threshold_value: Option<NumericalValue>) -> PropagatingProcess<O, S, C>`
- [x] 6.3 Stateful per-item fold replacing `PropagatingEffect<...>` with `PropagatingProcess<..., S, C>` throughout, calling `T::evaluate_stateful` on each item, threading the accumulator's `state` and `context` forward
- [x] 6.4 Reuse `monadic_collection_utils::aggregate_effects` unchanged for the final aggregation; lift its result into a `PropagatingProcess<O, S, C>` with the accumulated `state`, `context`, and `logs`
- [x] 6.5 Module-level and method-level rustdoc captured (relationship to stateless trait, explicit pointer to `from_causal_collection_with_context`, no stateful constructor exists)
- [x] 6.6 Register the new submodule in `deep_causality/src/traits/causable_collection/collection_reasoning/mod.rs`

## 7. StatefulMonadicCausableGraphReasoning trait and impl

- [x] 7.1 Create file `deep_causality/src/traits/causable_graph/graph_reasoning/stateful.rs`
- [x] 7.2 Trait defined with `Causaloid<V, V, S, C>: MonadicCausable<V, V> + StatefulMonadicCausable<V, V, S, C>` in the where-clause
- [x] 7.3 Three default-method implementations with stateful signatures
- [x] 7.4 Mirrors existing BFS / shortest-path bodies, using `causaloid.evaluate_stateful(...)`, threading `state` and `context` through the queue
- [x] 7.5 `RelayTo` branch lifts the `Box<PropagatingEffect<V>>` inner effect into a `PropagatingProcess<V, S, C>` carrying the relaying node's `state` and `context` at moment of relay
- [x] 7.6 Module-level rustdoc captured (relationship to stateless trait, explicit pointer to `from_causal_graph_with_context`, no stateful constructor exists)
- [x] 7.7 Register the new submodule in `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs`

- [x] 8.1 In `deep_causality/src/lib.rs`, add `pub use crate::traits::causable::stateful::StatefulMonadicCausable;` adjacent to the existing `MonadicCausable` re-export
- [x] 8.2 Re-export `StatefulMonadicCausableCollection` adjacent to the existing collection re-export
- [x] 8.3 Re-export `StatefulMonadicCausableGraphReasoning` adjacent to the existing graph reasoning re-export

## 8. Public re-exports

(completed inline alongside section 5/6/7 to satisfy compile dependencies; see boxes above)

## 9. Tests — singleton stateful evaluation

- [x] 9.1 Create file `deep_causality/tests/types/causal_types/causaloid/causable_stateful_tests.rs`
- [x] 9.2 Test `evaluate_stateful_threads_state_and_context_through_closure` — counter increments, state ≠ default, value reflects context multiplier
- [x] 9.3 Test `evaluate_stateful_passes_state_through_when_closure_is_stateless` — stateless `Causaloid::new` does not perturb caller state
- [x] 9.4 Test `evaluate_stateful_short_circuits_with_state_preserved_on_error` — error path keeps caller state intact, logs preserved
- [x] 9.5 Test `stateless_evaluate_unchanged_for_existing_callers` — regression guard
- [x] 9.6 Test `same_causaloid_evaluable_via_both_evaluate_and_evaluate_stateful` — same value, both paths
- [x] 9.7 Test file registered in `tests/types/causal_types/causaloid/mod.rs`; mod chain already present

## 10. Tests — collection stateful evaluation

- [x] 10.1 Create file `deep_causality/tests/traits/causable_collection/collection_reasoning/stateful_monadic_collection_tests.rs`
- [x] 10.2 Test `evaluate_collection_stateful_aggregates_and_threads_state` — three increments threaded across items + Some(2)-of-3 aggregation
- [x] 10.3 Test `evaluate_collection_stateful_short_circuits_with_state_at_failure_point` — failing item 2 leaves state at item 1's mutation; item 3 does not execute
- [x] 10.4 New mod chain registered: `tests/traits/causable_collection/{mod.rs, collection_reasoning/{mod.rs, stateful_monadic_collection_tests.rs}}`, parent `tests/traits/mod.rs` declares `pub mod causable_collection;`

- [x] 11.1 Create file `deep_causality/tests/traits/causable_graph/graph_reasoning/stateful_tests.rs`
- [x] 11.2 Test `evaluate_subgraph_from_cause_stateful_threads_state_across_three_nodes` — count=3 after BFS over a 3-node path
- [x] 11.3 Test `evaluate_subgraph_stateful_short_circuits_on_node_error` — failing node halts traversal, state at moment of failure preserved
- [x] 11.4 Test `evaluate_subgraph_stateful_relayto_preserves_state` — relayer node mutates state to 1, target observes it and increments to 2; intermediate node not executed
- [x] 11.5 Test `evaluate_single_cause_stateful_works` and `evaluate_shortest_path_between_causes_stateful_works`
- [x] 11.6 New mod chain registered: `tests/traits/causable_graph/{mod.rs, graph_reasoning/{mod.rs, stateful_tests.rs}}`, parent `tests/traits/mod.rs` declares `pub mod causable_graph;`

## 11. Tests — graph stateful evaluation

- [ ] 11.1 Create file `deep_causality/tests/traits/causable_graph/graph_reasoning/stateful_tests.rs`
- [ ] 11.2 Add a test that builds a frozen `CausaloidGraph` of three nodes wired in a path, each node's closure increments the counter field on `S`, and verifies that `evaluate_subgraph_from_cause_stateful` returns a process whose `state` reflects three increments and whose `logs` contain entries from all three nodes in traversal order
- [ ] 11.3 Add a test that triggers an error on the second node and verifies short-circuit + state preservation as in the collection case
- [ ] 11.4 Add a test for `RelayTo` with non-trivial state: a node emits `PropagatingEffect::RelayTo(target_index, inner_effect)` while carrying a non-trivial `state`, and the test verifies the relayed-to node observes that `state` and the final returned process reflects the relayed-to node's state mutation
- [ ] 11.5 Add a test for `evaluate_single_cause_stateful` and a test for `evaluate_shortest_path_between_causes_stateful`, each confirming the basic state-threading guarantee
- [ ] 11.6 Register the new test file in the corresponding `mod.rs` chain

## 12. Tests — backward-compatibility regression guard

- [x] 12.1 Existing `cargo test -p deep_causality` suite continues to pass — 1013 integration tests + 24 unit tests + doctests all green; no existing source files modified beyond the additive submodule registrations
- [x] 12.2 Stateless trait method signatures inspected and confirmed unchanged in `traits/causable/mod.rs`, `traits/causable_collection/.../monadic_collection.rs`, `traits/causable_graph/graph_reasoning/mod.rs`; only additions are submodule declarations

## 13. Verification

- [x] 13.1 `cargo build -p deep_causality` — zero errors, zero warnings
- [x] 13.2 `cargo test -p deep_causality` — all existing and new tests pass (1013 integration + 24 unit)
- [x] 13.3 `cargo clippy -p deep_causality --all-targets -- -D warnings` — clean
- [x] 13.4 `cargo fmt --check -p deep_causality` — clean (one fmt pass applied)
- [x] 13.5 Only the `deep_causality` crate was modified — single-crate verification sufficient
- [x] 13.6 Zero `unsafe`, zero `macro_rules!` in any new or modified source file under `deep_causality/src/`
- [x] 13.7 Zero new entries under `[dependencies]` in any `Cargo.toml`

## 14. Commit handoff

- [x] 14.1 Commit message prepared in conversation; per AGENTS.md golden rule #1, NOT committed — user to perform `git commit`
- [x] 14.2 Follow-up change `add-stateful-flight-envelope-example` is now implementable; its proposal/spec/tasks should be revised to use the new traits before re-applying
