## Context

The framework already exposes a stateful effect type
`PropagatingProcess<T, S, C> = CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>`
([deep_causality_core/src/types/propagating_process/mod.rs](deep_causality_core/src/types/propagating_process/mod.rs))
and the closure type
`ContextualCausalFn<I, O, S, C> = fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>`
([deep_causality/src/alias/alias_function.rs:39-40](deep_causality/src/alias/alias_function.rs#L39-L40)).
A `Causaloid<I, O, S, C>` carries this closure as
`context_causal_fn: Option<ContextualCausalFn<I, O, S, C>>`.

The four pieces that close the gap to actually-stateful evaluation are:

1. The trait method `MonadicCausable::evaluate` returns `PropagatingEffect<O>`
   (i.e. `S = (), C = ()`)
   ([traits/causable/mod.rs:26](deep_causality/src/traits/causable/mod.rs#L26)).
2. The helper `execute_causal_logic` invokes the closure with `PS::default()`
   and discards the resulting `state` field while converting the
   `PropagatingProcess` to `PropagatingEffect`
   ([types/causal_types/causaloid/causable_utils.rs:41-56](deep_causality/src/types/causal_types/causaloid/causable_utils.rs#L41-L56)).
3. The `MonadicCausable` impl for `Causaloid` only handles `Singleton` and
   returns explicit "use specialized APIs" errors for `Collection` and
   `Graph`, but the specialized collection / graph traits also return
   stateless `PropagatingEffect`
   ([types/causal_types/causaloid/causable.rs:74-127](deep_causality/src/types/causal_types/causaloid/causable.rs#L74-L127)).
4. Graph reasoning trait methods take and return `PropagatingEffect<V>`
   ([traits/causable_graph/graph_reasoning/mod.rs:40-61](deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs#L40-L61)).

This change extends each of these four points with a stateful sibling without
modifying the existing stateless surface. The motivation is to enable the
follow-up example `add-stateful-flight-envelope-example`, which requires
state-threading across a Causaloid → CausalMonad → Causaloid composition.

## Goals / Non-Goals

**Goals:**

- Provide a public, additive stateful evaluation API for all three causal
  forms: singleton, collection, and graph. A user with a
  `Causaloid<I, O, FlightState, AircraftConfig>` and a starting
  `PropagatingProcess<I, FlightState, AircraftConfig>` SHALL be able to
  evaluate it without losing `FlightState` or `AircraftConfig`.
- Preserve the closure layer's existing semantics. The new path threads
  whatever `state` and `context` the caller supplies into the closure, and
  returns whatever `state` the closure produces — no defaulting, no
  discarding.
- Preserve the stateless API as-is. Existing callers, examples, and tests
  continue to compile and behave identically. No symbol is renamed or
  removed.
- Match log-aggregation and error-short-circuit semantics of the stateless
  API exactly. Logs accumulate; errors halt downstream evaluation while
  preserving accumulated logs and the `state` at the moment of failure.
- Keep ergonomics in line with the existing trait surface: type-parameter
  count and naming conventions follow the existing `MonadicCausable*`
  traits (`<I, O, S, C>`, `<I, O, S, C, T>`, `<V, S, C>`).
- Honor AGENTS.md conventions: no `unsafe`, no macros in lib code, no new
  external dependencies, idiomatic zero-cost abstractions, static dispatch,
  one type / one module, errors / traits / types under their own files.

**Non-Goals:**

- The flight-envelope-monitor example itself. That belongs to the follow-up
  change `add-stateful-flight-envelope-example`, which becomes implementable
  once this change lands.
- Deprecating or rewriting the stateless `MonadicCausable*` traits. They
  remain the primary surface for users who do not need stateful evaluation,
  and future deprecation (if ever) is a separate decision.
- Adding async, parallel, or `tokio`-based evaluation paths.
- Modifying the underlying HKT machinery in `deep_causality_haft`. The
  `MonadEffect5` trait and `Functor` witnesses already accommodate the
  stateful process type via its existing `Effect5` / `MonadEffect5` impls.
- Introducing default type parameters (`S = ()`, `C = ()`) on the existing
  `MonadicCausable` trait. See Decision 1.

## Decisions

### Decision 1: New parallel traits, not default type parameters on the existing traits

**Choice**: Introduce three new traits — `StatefulMonadicCausable<I, O, S, C>`,
`StatefulMonadicCausableCollection<I, O, S, C, T>`, and
`StatefulMonadicCausableGraphReasoning<V, S, C>` — alongside the existing
`MonadicCausable<I, O>`, `MonadicCausableCollection<I, O, T>`, and
`MonadicCausableGraphReasoning<V, PS, C>`.

**Alternatives considered**:

1. **Add default type parameters to the existing trait**, e.g.
   `pub trait MonadicCausable<I, O, S = (), C = ()> { fn evaluate(...) -> PropagatingProcess<O, S, C>; }`
   Then `evaluate(...) -> PropagatingEffect<O>` becomes the special case
   `S = (), C = ()`. **Rejected**: Default type parameters in trait declarations
   are stable Rust but interact subtly with type inference at every call site.
   The existing trait is consumed throughout the crate and across user code;
   silently changing its return type from `PropagatingEffect<O>` to
   `PropagatingProcess<O, (), ()>` (which is a strict alias) is technically
   a no-op at runtime but produces churn in error messages, generated docs,
   and downstream rustdoc. It also opens the question of how to migrate
   existing impl blocks. The risk is greater than the elegance is worth.

2. **Replace the existing trait entirely** with the stateful one. **Rejected**:
   Breaking change. Forces every existing call site and impl block to pick
   up the new type parameters even when they only need stateless evaluation.
   AGENTS.md golden rule: surgical, additive change.

3. **Make the stateless trait a blanket impl over the stateful trait**, e.g.
   `impl<T, I, O> MonadicCausable<I, O> for T where T: StatefulMonadicCausable<I, O, (), ()>`.
   **Rejected**: The existing `MonadicCausable` impl for `Causaloid<I, O, PS, C>`
   would conflict with the blanket impl unless `PS = ()` and `C = ()`,
   forcing a coherence headache. Cleaner to keep the two trait families
   independent and let `Causaloid` impl both.

**Why parallel traits win**: Zero blast radius on existing code paths. Clear
semantics — the trait name says exactly what it does. The cost is two trait
families instead of one, but that cost is paid in one place (the crate root
re-exports) and is justified by the safety of additive change.

### Decision 2: Stateful method names use the `_stateful` suffix

**Choice**: `evaluate_stateful`, `evaluate_collection_stateful`,
`evaluate_single_cause_stateful`, `evaluate_subgraph_from_cause_stateful`,
`evaluate_shortest_path_between_causes_stateful`.

**Alternatives considered**:

1. `evaluate_with_state`. **Rejected**: Implies the caller passes a state
   separately, but in the proposed signature the state is carried inside
   the incoming `PropagatingProcess`. The suffix would mislead.
2. Drop the suffix and rely on the type signature.
   **Rejected**: The two methods on the same struct (`Causaloid` impls both
   `MonadicCausable::evaluate` and `StatefulMonadicCausable::evaluate_*`)
   need distinguishable names because both are reachable via UFCS without
   the trait being explicit; the suffix removes a real ambiguity.
3. `eval_s` / `eval_stf`. **Rejected**: AGENTS.md prefers idiomatic Rust;
   abbreviations hurt readability.

### Decision 3: The stateful method takes `&PropagatingProcess<I, S, C>` and returns `PropagatingProcess<O, S, C>`

**Choice**: Mirror the stateless signature shape — take the incoming process
by reference, return a new process by value — but use the stateful alias
`PropagatingProcess<T, S, C>` instead of `PropagatingEffect<T>`.

**Rationale**:

- `PropagatingProcess<T, S, C>` is the existing public alias for
  `CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>` — no
  new alias is needed.
- Taking `&PropagatingProcess<I, S, C>` is consistent with the stateless
  `evaluate(&self, &PropagatingEffect<I>)` and lets the implementation clone
  what it needs (state, context, value) without forcing the caller to give
  up ownership.
- Returning a new process by value matches the stateless API and the
  monadic semantics of `bind`.

**Alternatives considered**:

1. Take `PropagatingProcess<I, S, C>` by value. **Rejected**: forces caller
   to clone or move; inconsistent with stateless API.
2. Take a tuple `(value: I, state: S, context: Option<C>)` and return a
   tuple. **Rejected**: drops the log and error channels, defeats the
   purpose.

### Decision 4: `execute_causal_logic_stateful` is added as a sibling, original is unchanged

**Choice**: Add a new helper `execute_causal_logic_stateful<I, O, S, C>` in
the same file
([types/causal_types/causaloid/causable_utils.rs](deep_causality/src/types/causal_types/causaloid/causable_utils.rs))
that takes `(input: I, state: S, context: Option<C>, causaloid: &Causaloid<I, O, S, C>)`
and returns `PropagatingProcess<O, S, C>`. The original `execute_causal_logic`
is left untouched so the existing stateless evaluate continues to behave
identically.

**Rationale**:

- The two helpers serve fundamentally different contracts: one defaults
  state and projects to `PropagatingEffect`, the other threads state and
  preserves it. Trying to share implementation through a generic helper
  would muddy both contracts and force conditional logic.
- AGENTS.md: minimum necessary code, surgical diffs. Two short, focused
  helpers in the same file is simpler than one polymorphic helper.

**Alternatives considered**:

1. Refactor `execute_causal_logic` to be generic over a "preserve state"
   flag. **Rejected**: branchy, harder to read, and the two paths produce
   different return types (which would force trait gymnastics).
2. Inline the stateful helper into the new singleton trait impl.
   **Rejected**: a stateful collection or graph evaluation may want to call
   it too; keeping it as a `pub(super)` (or `pub(crate)`) helper preserves
   reuse.

### Decision 5: Stateful collection evaluation reuses `monadic_collection_utils::aggregate_effects` with a state-preserving wrapper

**Choice**: Implement `evaluate_collection_stateful` by folding over the
collection items with `CausalMonad::pure(initial_state, initial_context)`
applied to an empty `Vec<EffectValue<O>>`, calling
`evaluate_stateful` on each item, then delegating to the existing
`monadic_collection_utils::aggregate_effects` helper for the final
aggregation step. The accumulated state and context are preserved in the
output process; the existing aggregation helper is unchanged.

**Rationale**:

- The aggregation logic (`AggregateLogic`, threshold semantics) is
  identical between stateless and stateful — only the surrounding
  state-threading changes. Reusing the helper avoids duplication.
- The existing stateless implementation already uses `bind` on a
  `PropagatingProcess`-shaped accumulator (with `S = (), C = ()`); the
  stateful implementation is the same code with non-trivial `S` and `C`.

### Decision 6: Stateful graph reasoning reuses BFS topology with the new evaluate

**Choice**: `StatefulMonadicCausableGraphReasoning::evaluate_subgraph_from_cause_stateful`
performs the same BFS traversal as its stateless counterpart but invokes
`evaluate_stateful` on each node, threading the resulting state and context
into the next node's incoming process. The `RelayTo` adaptive-reasoning
behavior is preserved.

**Rationale**: Same as Decision 5. The traversal logic is unchanged; only the
per-node evaluation primitive switches.

### Decision 7: Error semantics — state at moment of failure is preserved on the returned process

**Choice**: When a node returns an error, the stateful evaluator returns the
new process with `error: Some(err)`, `value: EffectValue::default()`, and
**`state` set to the state carried at the moment of failure** (i.e. the
state of the failing node's incoming process). Logs accumulated up to and
including the failing node are preserved.

**Rationale**:

- The proposal's spec asserts this guarantee. It matches the
  `CausalEffectPropagationProcess::bind` short-circuit semantics
  ([deep_causality_core/src/types/causal_effect_propagation_process/mod.rs:75-92](deep_causality_core/src/types/causal_effect_propagation_process/mod.rs#L75-L92))
  which already preserves `state` on error.
- Defaulting state on error would lose the very thing the stateful API
  exists to preserve. Doing so silently would surprise the caller.

### Decision 8: Module placement and naming follow the existing convention

**Choice**:

- New trait per file:
  `traits/causable/stateful.rs`,
  `traits/causable_collection/collection_reasoning/stateful_monadic_collection.rs`,
  `traits/causable_graph/graph_reasoning/stateful.rs`.
- New impl per file:
  `types/causal_types/causaloid/causable_stateful.rs`.
- Test files mirror the source tree per AGENTS.md:
  `tests/traits/causable/stateful_tests.rs`,
  `tests/traits/causable_collection/collection_reasoning/stateful_monadic_collection_tests.rs`,
  `tests/traits/causable_graph/graph_reasoning/stateful_tests.rs`,
  `tests/types/causal_types/causaloid/causable_stateful_tests.rs`.

**Rationale**: AGENTS.md "One type, one Rust module" and the test-folder mirror
convention. No deviation.

### Decision 9: Re-export the three new traits at crate root

**Choice**: Add three lines to `deep_causality/src/lib.rs` next to the
existing `MonadicCausable*` re-exports, so consumers can write
`use deep_causality::StatefulMonadicCausable;` etc.

**Rationale**: AGENTS.md requires public symbols be exported from `lib.rs`
and consumed via crate-root imports.

### Decision 10: No new external dependencies; no `unsafe`; no macros

**Choice**: Implementation uses only existing workspace dependencies
(`deep_causality_core`, `deep_causality_haft`). No external crates.
No `unsafe`. No macros in lib code.

**Rationale**: AGENTS.md hard constraints.

### Decision 11: Add a `StatefulContextualCausalFn` type alias; do NOT add new `Causaloid` constructors

**Choice**: Two concrete additions and one explicit non-addition:

1. **Add** a new public type alias in
   `deep_causality/src/alias/alias_function.rs`:
   ```rust
   pub type StatefulContextualCausalFn<I, O, S, C> =
       fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;
   ```
   Structurally identical to `ContextualCausalFn` (same `fn` pointer
   type at the type-system level); a clearly-named ergonomic marker at
   the closure-author site.
2. **Add** rustdoc on each new trait method
   (`evaluate_stateful`, `evaluate_collection_stateful`, the three
   stateful graph-reasoning methods) explicitly directing the reader
   to: (a) the existing `_with_context` constructors on `Causaloid`,
   and (b) the `StatefulContextualCausalFn` alias as the recommended
   way to declare closures intended for the stateful evaluation path.
3. **Do NOT add** any new `Causaloid` constructor (no
   `new_stateful_with_context`, no
   `from_causal_collection_stateful_with_context`, no
   `from_causal_graph_stateful_with_context`). Stateful evaluation is
   determined entirely by which trait method the caller invokes; no
   existing `Causaloid` constructor makes a state-threading decision.

**Rationale for the alias**:

- The existing `ContextualCausalFn<I, O, S, C>` already returns
  `PropagatingProcess<O, S, C>` — i.e. its shape is already stateful.
  A user reading the API surface today cannot tell from the alias name
  alone whether the function carries state through evaluation; they
  must read the rustdoc on `Causaloid::new_with_context` and
  cross-reference `MonadicCausable::evaluate` to discover that the
  state is silently dropped at the trait-method boundary.
- Introducing `StatefulContextualCausalFn` as a distinct, clearly-named
  alias makes the user's intent explicit at the closure-author site.
  The name aligns with the new `StatefulMonadicCausable` trait so that
  closure type and trait method form a recognizable pair.

**Rationale for not adding new constructors**:

- The two aliases (`ContextualCausalFn` and `StatefulContextualCausalFn`)
  resolve to the **same `fn` pointer type**. A constructor that took
  one and stored it in the same field as a constructor that took the
  other would be byte-for-byte identical at runtime and at the
  type-system level. The constructor would not enforce any guarantee
  the existing constructor doesn't already enforce.
- The same `Causaloid` value can be evaluated *either* statelessly
  (via `MonadicCausable::evaluate`) *or* statefully (via
  `StatefulMonadicCausable::evaluate_stateful`). Statefulness is a
  property of the **call**, not of the **constructor**. Naming a
  constructor "stateful" implies a guarantee that the call site can
  trivially violate.
- Symmetry across causal forms matters. The collection and graph
  constructors don't accept closures at all; their state-threading is
  determined by their children. Adding a "stateful" naming variant
  there would be even more clearly cosmetic. Adding it only on the
  singleton form (an asymmetry I rejected after questioning during
  spec review) would teach users a pattern that doesn't generalize.
- AGENTS.md "Make Surgical Diffs": the smallest API surface that
  achieves the goal. The goal — clarity at the call site for stateful
  evaluation — is achieved by the type alias and by trait-method
  rustdoc. A new constructor would be net surface area without net
  clarity.

**Alternatives considered**:

1. **Add `new_stateful_with_context` for the singleton form only.**
   **Rejected** during spec review. The singleton "stateful"
   constructor would store the same data in the same fields as
   `new_with_context` (the two closure-type aliases resolve to the
   same `fn` pointer), so the constructor name would be a pure
   documentation marker. Adding it only for singletons is asymmetric
   without a principled reason; adding it for all three forms creates
   even more naming-only surface area without the closure-author
   ergonomic argument that motivated the singleton case.

2. **Add `new_stateful_with_context`, `from_causal_collection_stateful_with_context`, and `from_causal_graph_stateful_with_context` for symmetry.**
   **Rejected**: six constructors instead of three for zero
   structural difference. The "stateful" naming on a constructor that
   does not control statefulness would be misleading: a user calling
   `from_causal_collection_stateful_with_context(...)` then
   `evaluate_collection(...)` (stateless) gets stateless behavior,
   but the constructor name suggested otherwise. Future-proofing
   ("if the framework later differentiates stateful from stateless at
   the type level…") is not a strong enough reason to ship six
   constructors today; adding new constructors later is itself an
   additive change.

3. **Make `StatefulContextualCausalFn` a transparent
   `pub type X = ContextualCausalFn<...>` re-alias rather than a
   spelled-out `fn(...)` definition.** **Rejected**: a transparent
   re-alias points a reader from one alias to another; spelling the
   function shape directly is stronger self-documentation. The two
   forms resolve to the same `fn` pointer type, so there is no
   compile-time or runtime difference; only the rustdoc surface
   differs.

**Trade-off**: Two type aliases that resolve to the same underlying
`fn` pointer is mild duplication. Cost: ~10 lines including rustdoc
in `alias_function.rs`. Benefit: a clear, name-matched closure type
for users authoring code on the stateful evaluation path. The trade
is paid in one file and lifted at every closure-author site.

**Honesty note**: Statefulness on a `Causaloid` is a property of the
**evaluation call**, not of the constructor. The new traits and the
new alias make this explicit; the deliberate absence of new
constructors makes it inarguable.

## Risks / Trade-offs

- **Risk**: Two parallel trait families (`MonadicCausable*` and
  `StatefulMonadicCausable*`) increase the public surface area and may
  confuse new users who don't know which to use. → **Mitigation**: The
  rustdoc on each new trait opens with a one-paragraph "When to use this
  vs. `MonadicCausable<...>`" decision guide. The crate-level docs in
  `lib.rs` reference both families with the same guidance. The
  flight-envelope-monitor example (follow-up change) demonstrates the
  stateful path concretely.

- **Risk**: The blanket-impl alternative (Decision 1, alternative 3) might
  later look more attractive once both APIs have settled, and migrating to
  it would be a non-trivial coherence change. → **Mitigation**: This is
  acceptable because adding a blanket impl in the future is itself
  additive. Choosing parallel traits now does not foreclose future
  consolidation.

- **Risk**: Stateful evaluation can hide subtle bugs where state is mutated
  during a step that later short-circuits on error, leaving the caller with
  partially-mutated state. → **Mitigation**: This is a feature, not a bug —
  Decision 7 is explicit about preserving state at moment of failure. The
  rustdoc on the trait method states this precisely so callers can reason
  about it. Tests cover the failing-step state-preservation case.

- **Trade-off**: The stateful API is strictly more flexible than the
  stateless one (any stateless evaluation can be expressed by passing
  `S = (), C = ()`), but we are not deduplicating them at this point. The
  cost is two implementations of essentially-the-same evaluation logic.
  This is intentional — see Decision 1 — and the duplication is bounded
  (one impl block per causal form, three forms, two trait families = six
  evaluators, all in three new files).

- **Risk**: Test coverage may miss edge cases such as: a context-aware
  closure that mutates state, a context-aware closure that returns an
  intermediate `EffectValue::None`, a graph node that emits `RelayTo` while
  the caller carries non-trivial state. → **Mitigation**: The spec and
  tasks call out these specific scenarios. The test plan includes
  property-style cases for state evolution and error short-circuit at
  every causal form.

- **Risk**: AGENTS.md golden rule #2 forbids deleting files. This change
  only adds files; no deletion is needed. → **Mitigation**: Spelled out
  here for verification: zero files are deleted by this change.
