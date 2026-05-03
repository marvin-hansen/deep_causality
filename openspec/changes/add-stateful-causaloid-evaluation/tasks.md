## 1. Pre-implementation reading

- [ ] 1.1 Re-read `proposal.md`, `design.md`, and `specs/stateful-causaloid-evaluation/spec.md` for this change before writing any code
- [ ] 1.2 Re-read `AGENTS.md` to confirm conventions (one type / one module, tests mirror source tree, no `unsafe`, no macros in lib, idiomatic zero-cost abstractions, no new external crates)
- [ ] 1.3 Re-read `deep_causality/src/types/causal_types/causaloid/causable_utils.rs` and `deep_causality/src/types/causal_types/causaloid/causable.rs` to anchor on existing implementation patterns
- [ ] 1.4 Re-read `deep_causality_core/src/types/propagating_process/mod.rs` and `deep_causality_core/src/types/causal_effect_propagation_process/mod.rs` to confirm the public API of `PropagatingProcess` and the existing `bind` semantics on the underlying process type

## 2. Stateful closure type alias

- [ ] 2.1 In `deep_causality/src/alias/alias_function.rs`, add a new public type alias `pub type StatefulContextualCausalFn<I, O, S, C> = fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;` adjacent to the existing `ContextualCausalFn`; do not modify `ContextualCausalFn`
- [ ] 2.2 Add rustdoc on `StatefulContextualCausalFn` explaining the relationship to `ContextualCausalFn` (same `fn` pointer shape, parallel name, recommended for closures intended to be evaluated via `StatefulMonadicCausable::evaluate_stateful` and similar stateful trait methods); the rustdoc SHALL also state that the existing `Causaloid::new_with_context` constructor accepts this alias and is the recommended path for authoring a Causaloid intended for stateful evaluation — no new constructor exists or is needed
- [ ] 2.3 In `deep_causality/src/lib.rs`, re-export the new alias: `pub use crate::alias::alias_function::StatefulContextualCausalFn;` adjacent to the existing `ContextualCausalFn` re-export (if there is no existing re-export of `ContextualCausalFn` at crate root, then verify the new alias surfaces via the existing `pub use crate::alias::*;` glob)
- [ ] 2.4 Confirm by inspection that `deep_causality/src/types/causal_types/causaloid/mod.rs` has zero added `pub fn` items in any `impl` block on `Causaloid` after this change (i.e. no new constructors are introduced)

## 3. Stateful execute_causal_logic helper

- [ ] 3.1 Add a new function `execute_causal_logic_stateful<I, O, S, C>(input: I, state: S, context: Option<C>, causaloid: &Causaloid<I, O, S, C>) -> PropagatingProcess<O, S, C>` to `deep_causality/src/types/causal_types/causaloid/causable_utils.rs` (same file, additive, do not touch `execute_causal_logic`)
- [ ] 3.2 In the new function, when `causaloid.context_causal_fn` is set, invoke it with the supplied `state` and `context` (do not call `S::default()`) and return the resulting `PropagatingProcess<O, S, C>` unchanged (no conversion, no log loss)
- [ ] 3.3 In the new function, when only `causaloid.causal_fn` is set, invoke the stateless closure on the value and lift the resulting `PropagatingEffect<O>` into a `PropagatingProcess<O, S, C>` whose `state` and `context` fields are the pass-through `state` and `context` passed in by the caller and whose `value`, `error`, `logs` come from the stateless effect
- [ ] 3.4 In the new function, when neither closure is set, return a `PropagatingProcess::from_error(...)` with a precise error message identifying the causaloid id, preserving the supplied `state` and `context` on the returned process
- [ ] 3.5 Export the new helper as `pub(super)` (same visibility as `execute_causal_logic`)

## 4. StatefulMonadicCausable trait

- [ ] 4.1 Create file `deep_causality/src/traits/causable/stateful.rs`
- [ ] 4.2 Define `pub trait StatefulMonadicCausable<I, O, S, C>` with one method `fn evaluate_stateful(&self, incoming: &PropagatingProcess<I, S, C>) -> PropagatingProcess<O, S, C>;`
- [ ] 4.3 Add module-level rustdoc explaining the relationship to `MonadicCausable<I, O>` (when to use which, that they coexist, that this one preserves `S` and `C`)
- [ ] 4.4 Add per-method rustdoc covering: state-threading semantics, log aggregation, error short-circuit semantics (state at moment of failure is preserved), and the pass-through behavior when the underlying causaloid uses the stateless `causal_fn`
- [ ] 4.5 In the rustdoc for `evaluate_stateful`, explicitly direct the reader to the existing `Causaloid::new_with_context` constructor as the way to author a Causaloid intended for stateful evaluation, and to the `StatefulContextualCausalFn` type alias as the recommended type for the closure argument; state that statefulness is a property of the **call** (which trait method is invoked), not of the **constructor**
- [ ] 4.6 Register the new submodule in `deep_causality/src/traits/causable/mod.rs` (add `pub mod stateful;` next to the existing module declarations); do not modify the existing `MonadicCausable` trait

## 5. Causaloid impl of StatefulMonadicCausable

- [ ] 5.1 Create file `deep_causality/src/types/causal_types/causaloid/causable_stateful.rs`
- [ ] 5.2 Add `use` lines for the new trait, `Causaloid`, `CausaloidType`, `CausalityError`, `PropagatingProcess`, `EffectValue`, `causable_utils::execute_causal_logic_stateful`, and any helper macros from `deep_causality_haft` already in use by `causable.rs` (notably `LogAddEntry`)
- [ ] 5.3 Implement `StatefulMonadicCausable<I, O, S, C> for Causaloid<I, O, S, C>` with the same trait bounds as the existing `MonadicCausable` impl (`I: Default + Clone + Send + Sync + 'static + Debug`, `O: Default + Debug + Clone + Send + Sync + 'static`, `S: Default + Clone + Send + Sync + 'static`, `C: Clone + Send + Sync + 'static`)
- [ ] 5.4 Inside `evaluate_stateful`, branch on `self.causal_type`:
  - `CausaloidType::Singleton`: chain log_input_stateful → execute_causal_logic_stateful → log_output_stateful using the existing `bind` on `PropagatingProcess` (use the `bind` defined on `CausalEffectPropagationProcess` in `deep_causality_core`); thread `state` and `context` from `incoming` end-to-end
  - `CausaloidType::Collection`: return a `PropagatingProcess::from_error(...)` mirroring the existing stateless behavior, with a message directing the caller to `evaluate_collection_stateful`; preserve incoming `state`, `context`, and `logs`
  - `CausaloidType::Graph`: same as above but directing the caller to `evaluate_subgraph_from_cause_stateful`
- [ ] 5.5 Add stateful versions of `log_input` and `log_output` if needed (e.g. `log_input_stateful`, `log_output_stateful`) inside the impl module or in `causable_utils.rs`; reuse the same log message format strings as the stateless helpers so log entries remain consistent in shape
- [ ] 5.6 Register the new submodule in `deep_causality/src/types/causal_types/causaloid/mod.rs` next to `mod causable;` and `mod causable_utils;`

## 6. StatefulMonadicCausableCollection trait and impl

- [ ] 6.1 Create file `deep_causality/src/traits/causable_collection/collection_reasoning/stateful_monadic_collection.rs`
- [ ] 6.2 Define `pub trait StatefulMonadicCausableCollection<I, O, S, C, T>: CausableCollectionAccessor<I, O, T>` with method `fn evaluate_collection_stateful(&self, incoming: &PropagatingProcess<I, S, C>, logic: &AggregateLogic, threshold_value: Option<NumericalValue>) -> PropagatingProcess<O, S, C>`
- [ ] 6.3 Mirror the existing `evaluate_collection` body, replacing `PropagatingEffect<...>` with `PropagatingProcess<..., S, C>` throughout, calling `T::evaluate_stateful` on each item, and threading the accumulator's `state` and `context` through each `bind`
- [ ] 6.4 Reuse `monadic_collection_utils::aggregate_effects` unchanged for the final aggregation; lift its result into a `PropagatingProcess<O, S, C>` with the accumulated `state`, `context`, and `logs`
- [ ] 6.5 Add module-level rustdoc explaining the relationship to `MonadicCausableCollection<I, O, T>`; in the rustdoc for `evaluate_collection_stateful`, direct the reader to the existing `Causaloid::from_causal_collection_with_context` constructor and note that no "stateful" sibling constructor exists or is needed (statefulness is selected by calling this trait method instead of the stateless one)
- [ ] 6.6 Register the new submodule in `deep_causality/src/traits/causable_collection/collection_reasoning/mod.rs`

## 7. StatefulMonadicCausableGraphReasoning trait and impl

- [ ] 7.1 Create file `deep_causality/src/traits/causable_graph/graph_reasoning/stateful.rs`
- [ ] 7.2 Define `pub trait StatefulMonadicCausableGraphReasoning<V, S, C>: CausableGraph<Causaloid<V, V, S, C>>` with the trait bounds matching the existing `MonadicCausableGraphReasoning<V, PS, C>` (substituting `S` for `PS`) and add `Causaloid<V, V, S, C>: StatefulMonadicCausable<V, V, S, C>` to the where-clause
- [ ] 7.3 Define three default-method implementations with signatures:
  - `fn evaluate_single_cause_stateful(&self, index: usize, effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`
  - `fn evaluate_subgraph_from_cause_stateful(&self, start_index: usize, initial_effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`
  - `fn evaluate_shortest_path_between_causes_stateful(&self, start_index: usize, stop_index: usize, initial_effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`
- [ ] 7.4 Mirror the existing default-method bodies, replacing every `PropagatingEffect<V>` with `PropagatingProcess<V, S, C>`, every `causaloid.evaluate(...)` with `causaloid.evaluate_stateful(...)`, and ensuring `state` and `context` are threaded through the BFS via the accumulator process
- [ ] 7.5 Preserve the `RelayTo` adaptive-jump branch exactly: when a node returns `PropagatingEffect::RelayTo`, the relayed-to node receives a `PropagatingProcess` whose `state` and `context` are the ones carried by the relaying node at the moment of relay
- [ ] 7.6 Add module-level rustdoc explaining the relationship to `MonadicCausableGraphReasoning<V, PS, C>`; in the rustdoc for the three new methods, direct the reader to the existing `Causaloid::from_causal_graph_with_context` constructor and note that no "stateful" sibling constructor exists or is needed (statefulness is selected by calling these trait methods instead of the stateless ones)
- [ ] 7.7 Register the new submodule in `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs`

## 8. Public re-exports

- [ ] 8.1 In `deep_causality/src/lib.rs`, add `pub use crate::traits::causable::stateful::StatefulMonadicCausable;` adjacent to the existing `MonadicCausable` re-export
- [ ] 8.2 Add `pub use crate::traits::causable_collection::collection_reasoning::stateful_monadic_collection::StatefulMonadicCausableCollection;` adjacent to the existing collection re-export
- [ ] 8.3 Add `pub use crate::traits::causable_graph::graph_reasoning::stateful::StatefulMonadicCausableGraphReasoning;` adjacent to the existing graph reasoning re-export

## 9. Tests — singleton stateful evaluation

- [ ] 9.1 Create file `deep_causality/tests/types/causal_types/causaloid/causable_stateful_tests.rs` (per AGENTS.md test-mirror convention)
- [ ] 9.2 Add a test that builds a `Causaloid` via `Causaloid::new_with_context` whose closure (typed as `StatefulContextualCausalFn<I, O, S, C>`) increments a counter field on `S` and verifies that `evaluate_stateful` returns a process whose `state` reflects the increment and whose `state` is not `S::default()`
- [ ] 9.3 Add a test that builds a `Causaloid` with the stateless `causal_fn` variant (via `Causaloid::new`) and verifies that `evaluate_stateful` passes the incoming `state` and `context` through unchanged
- [ ] 9.4 Add a test that builds a `Causaloid` via `new_with_context` whose closure returns an error and verifies that the returned process has `error: Some(...)`, `state` equal by `PartialEq` to the incoming state, and at least one log entry preserved
- [ ] 9.5 Add a test that confirms the existing `MonadicCausable::evaluate` on the same `Causaloid` returns the same `value` and `logs` shape as before this change (regression guard)
- [ ] 9.6 Add a test that confirms the same `Causaloid` value (built once via `new_with_context`) can be evaluated both via `MonadicCausable::evaluate` and via `StatefulMonadicCausable::evaluate_stateful`, demonstrating that the two evaluation paths coexist on a single value (no constructor distinction is required)
- [ ] 9.7 Register the new test file in `deep_causality/tests/types/causal_types/causaloid/mod.rs` with `#[cfg(test)]`; create that mod file if it does not already exist, and ensure it is registered up the test module tree all the way to `tests/mod.rs`

## 10. Tests — collection stateful evaluation

- [ ] 10.1 Create file `deep_causality/tests/traits/causable_collection/collection_reasoning/stateful_monadic_collection_tests.rs`
- [ ] 10.2 Add a test that builds a three-item collection with a 2-of-3 threshold, where each item increments the counter field on `S`, and verifies the returned process's `state` reflects three increments AND the aggregation outcome equals what the stateless `evaluate_collection` produces on the same inputs
- [ ] 10.3 Add a test that triggers an error on the second of three items and verifies the returned process's `state` reflects only the first item's increment, `error` is `Some(...)`, and logs from items 1 and 2 are preserved while no log from item 3 is present
- [ ] 10.4 Register the new test file in the corresponding `mod.rs` chain per AGENTS.md

## 11. Tests — graph stateful evaluation

- [ ] 11.1 Create file `deep_causality/tests/traits/causable_graph/graph_reasoning/stateful_tests.rs`
- [ ] 11.2 Add a test that builds a frozen `CausaloidGraph` of three nodes wired in a path, each node's closure increments the counter field on `S`, and verifies that `evaluate_subgraph_from_cause_stateful` returns a process whose `state` reflects three increments and whose `logs` contain entries from all three nodes in traversal order
- [ ] 11.3 Add a test that triggers an error on the second node and verifies short-circuit + state preservation as in the collection case
- [ ] 11.4 Add a test for `RelayTo` with non-trivial state: a node emits `PropagatingEffect::RelayTo(target_index, inner_effect)` while carrying a non-trivial `state`, and the test verifies the relayed-to node observes that `state` and the final returned process reflects the relayed-to node's state mutation
- [ ] 11.5 Add a test for `evaluate_single_cause_stateful` and a test for `evaluate_shortest_path_between_causes_stateful`, each confirming the basic state-threading guarantee
- [ ] 11.6 Register the new test file in the corresponding `mod.rs` chain

## 12. Tests — backward-compatibility regression guard

- [ ] 12.1 Run the entire existing `cargo test -p deep_causality` suite and confirm every existing test passes with no source modifications. If any test fails, the failure indicates an unintended behavioral change to the stateless API and MUST be fixed by adjusting the new code, not the test
- [ ] 12.2 Run `cargo doc -p deep_causality --no-deps` before and after the change in two scratch worktrees and diff the generated docs for `MonadicCausable`, `MonadicCausableCollection`, and `MonadicCausableGraphReasoning`; confirm no functional changes to those entries

## 13. Verification

- [ ] 13.1 `cargo build -p deep_causality` — zero errors, zero warnings
- [ ] 13.2 `cargo test -p deep_causality` — all existing and new tests pass
- [ ] 13.3 `cargo clippy -p deep_causality --all-targets -- -D warnings` — clean
- [ ] 13.4 `cargo fmt --check -p deep_causality` — clean
- [ ] 13.5 If three or more crates were touched, also run `make build && make test && make format && make fix`; otherwise build and test the single affected crate
- [ ] 13.6 Grep the new and modified source files for `unsafe` and `macro_rules!` and confirm zero matches under `deep_causality/src/`
- [ ] 13.7 Grep all `Cargo.toml` files in the workspace and confirm no new crates.io dependency entries were introduced by this change

## 14. Commit handoff

- [ ] 14.1 Prepare a commit message summarizing the framework extension and listing all new and modified files. Per AGENTS.md golden rule #1, do not commit; ask the user to commit
- [ ] 14.2 Note in the handoff that the follow-up change `add-stateful-flight-envelope-example` is now implementable, and that its proposal/design/spec/tasks should be revised against this completed framework extension before applying
