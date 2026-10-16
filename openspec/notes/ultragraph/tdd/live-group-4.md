<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 4: the Live state, its transitions and freeze_clone

Change `ultragraph-live-state-maintained-views`, tasks 4.1 to 4.7, run on 2026-10-16. The five
phases of `openspec/specs/unified-math-tdd-protocol/spec.md`.

## Decisions settled at phase 1

- The transition into Live is `make_live()` (the design's open question named it `live`
  provisionally); `freeze` and `unfreeze` convert from Live in place as from the other states;
  `freeze_clone` returns a frozen copy from any state.
- `is_live()` is an inherent method of the container, beside `freeze` and `compact_ids`; the
  public `GraphView` trait is unchanged, so no implementor outside the crate is touched.
- The arenas are struct-of-arrays, `targets`, `weights` and `rows` per direction, both holding
  `W`: Frozen → Live adopts the two CSRs without a copy, O(V) for the rows, and Live → Frozen
  packs the rows back without a sort. The backward arena therefore carries the payload, as the
  backward CSR does, and a replacement writes both rows.
- `project` in the Live state goes through a crate-internal `Backing` enum inside
  `ProjectionView` (`projection_view/backing.rs`), the frozen CSR or the live arenas, so the
  view's adjacency, reads and traversals are written once.
- The mask is the node table: a removed node's entries stay in its neighbours' rows and every
  read skips them (design D1); `number_edges` stays exact because a removal subtracts the
  node's live degree, counting each edge once.

## Corner cases A–K

| # | Class | Instance | Test |
|---|---|---|---|
| A | Empty | An empty graph made live from Dynamic and from Frozen, mutable; every algorithm on it; freeze and unfreeze of it | `test_an_empty_graph_made_live_is_empty_and_mutable`, `test_an_empty_frozen_graph_made_live_is_empty` (`graph_live/graph_mut_tests.rs`); `test_algorithms_on_an_empty_live_graph` (`adjacency_tests.rs`); `test_freeze_of_an_empty_live_graph`, `test_unfreeze_of_an_empty_live_graph` |
| B | Single element | One node whose first edge, a self-loop, gives its rows a capacity of one | `test_one_node_takes_a_self_loop_in_a_row_of_capacity_one` |
| C | Coinciding quantities | A put of an existing key (a replacement) beside a put of a new key (an insertion); the accounting at a row of capacity one, where twice the capacity and length plus one coincide, beside capacity two, where they differ | `test_a_repeated_key_replaces_and_a_new_key_adds`; `test_a_rebuilt_row_grows_the_arena` and `test_a_full_row_doubles_its_capacity` (`tests/utils/utils_tests_tests.rs`) |
| D | Degenerate index | The last node's row growing at the arena's end; a row rebuilt at the end; a removed node whose entries remain in its neighbours' rows, and two removals in turn counting each edge once | `test_the_last_nodes_row_grows`, `test_a_row_that_outgrows_its_capacity_keeps_every_edge`, `test_a_removed_node_in_live_leaves_every_read`, `test_two_removals_count_each_edge_once` |
| E | Thresholds, both sides | A row at capacity, one below and one above, through capacities 1, 2, 4 and 8 with targets out of order | `test_a_row_that_outgrows_its_capacity_keeps_every_edge` |
| F | Zero | A weight of 0, stored, read and routed in Live; zero live nodes after removals, through the model drive | `test_a_zero_weight_is_an_edge_in_live`; the model drives |
| G | Negative | n/a: positions, lengths and capacities are `usize` | — |
| H | Exact boundary | A row filled to its capacity exactly, emptied by one and refilled in place | `test_a_removal_frees_a_slot_the_next_insert_takes` |
| I | Non-finite | n/a: payloads are never compared by the storage beyond their keys | — |
| J | Overflow reach | n/a: capacities double from a row's length, bounded by memory | — |
| K | Every payload kind | `()`, `u64`, an identity key `Rel` and a field key `Tagged`, each through the agreement drive, the weighted shortest path compared where the payload is addable | the four `test_live_agrees_with_a_freshly_frozen_graph_*` tests |

**Independent sources.** The agreement drive (`adjacency_tests.rs`) applies the same random
operations to a Live graph and to a Dynamic reference, clones and freezes the reference before
every comparison, and compares every read, both traversals, every pair's reachability and
shortest paths, the weighted path where defined, every structural, topological and centrality
algorithm (float scores compared bit for bit, since both run the same body over adjacencies
sorted the same way): the `Vec` lists and the CSR against the arenas, sharing no code below the
algorithm bodies. The model drive of `graph_evolve_tests.rs` carries a third state and checks
`freeze_clone` as a frozen copy of the model with the source left as it was. Hand-derived
fixtures carry the rest.

## Phase 1 (task 4.1)

`LiveGraph<N, W> { nodes, live_nodes, forward: Arena<W>, backward: Arena<W>, live_edges,
root_index }`, `Arena { targets, weights, rows }`, `Row { start, len, cap }` in
`src/types/storage/graph_live/mod.rs`; `GraphState::Live`; `GraphError::GraphIsLive`; the
`Adjacency`, `GraphView`, `GraphMut`, `GraphTraversal`, `Freezable`, `Unfreezable` impls and the
accounting with `unimplemented!()` bodies; the four algorithm traits delegating to the generic
bodies; `make_live`, `is_live` and `freeze_clone` on the container, `freeze` and `unfreeze`
converting from Live, a Live arm in every container dispatch, `TraversalIter::Live`, the
`Backing` enum of the projection view and `CsmGraph::into_parts`. `cargo build -p ultragraph`
passes; the 257 tests of group 3 pass on the surface.

## Phase 2 (task 4.3)

Fifty-five tests: six files under `tests/types/storage/graph_live/` (the Bazel suite
`graph_live_tests`), the extended model drive and four transition tests in
`graph_evolve_tests.rs`, `GraphIsLive` in `graph_compact_tests.rs` and `error_tests.rs`, two
projection tests in `graph_project_tests.rs`, four accounting tests in `utils_tests_tests.rs`.
Every scenario of `ultragraph-three-states` without a maintained view has a test; the three
scenarios that register a view belong to group 6.

Exit run: **52 failed, 258 passed** of 310, every failure the unimplemented panic in
`graph_live/mod.rs`, reached first through `make_live` by every Live test. (The two
audit-supporting tests, `test_a_full_row_doubles_its_capacity` and
`test_a_payload_replaced_in_live_is_read_from_both_rows_of_a_view`, were added before the audit
and after that run, which puts the suite at 312.)

The first run of the throwaway found three defects in the suite, none in the code: a round-trip
snapshot that compared the dynamic state's unspecified list order with the sorted live order,
an edge count of 8 where the operations give 7, and an exact size hint asserted on a traversal
the mask filters. The tests were corrected, not the implementation.

## Phase 3 (task 4.4)

A throwaway implementation, 20 defects by `audit.py` (scratchpad), the whole suite against each:

| Defect | Class | Tests failing |
|---|---|---:|
| L1 a row rebuilt without its spare capacity | named | 1, the capacity accounting test |
| L2 a relocated row's record not moved | named | 20 |
| L3 the mask bit not cleared on removal | named | 16 |
| L4 stale entries read through a cleared mask | named | 12 |
| L5 `freeze_clone` returns the source's state | named (changing the source) | 4 |
| L6 `compact_ids` in Live answers the frozen error | named (accepted in Live) | 4 |
| G1 a run starts after its first entry | off-by-one | 18 |
| G2 the insertion point among equal targets reversed | flipped comparison | 4 |
| G3 capacity tripled | constant factor | 1, the accounting test |
| G4 a removal shifts the targets, not the weights | early return | 4 |
| G5 an edge to a removed node accepted | guard removed | 8 |
| G6 a removal subtracts the outgoing edges alone | plausible neighbour | 10 |
| G7 the passing count reads the table's length | plausible neighbour | 4 |
| G8 the adopted edge count zeroed | plausible neighbour | 17 |
| G9 `freeze` counts stale entries in the offsets | off-by-one | 6 |
| G10 `unfreeze` keeps stale entries | guard removed | 3 |
| G11 `contains_edge` ignores the target's mask | guard removed | 9 |
| G12 a pair removal leaves the backward run | early return | 5 |
| G13 a replacement leaves the backward payload | early return | 1, the view's edge test on the backward row |
| G14 a keyed removal leaves the backward entry | early return | 7 |

No survivor. Tolerance: none. Input variety: row C pairs the capacity where doubling and
length-plus-one coincide with one where they differ. The throwaway was discarded (the phase-1
files restored, 54 failures observed: the 52 and the two tests added for the audit) before
phase 4.

## Phase 4 (task 4.5)

The audited throwaway re-applied. `cargo test -p ultragraph`: 312 passed; clippy clean (one
`type_complexity` lint resolved with the `CsmParts` alias, no `allow`); `bazel test //...`
green with the new suite. Coverage: every line of `graph_live/` and of the changed container
files is reached by a named test above; no coverage tool was run.

## Phase 5 (task 4.6)

`cargo mutants -p ultragraph --in-diff` over every source line the group added or changed
(`graph_live/*`, the container's dispatch and transitions, `TraversalIter`, the view's
`Backing`, `CsmGraph::into_parts`, the error variant): **139 mutants, 126 caught, 11 unviable,
2 timeouts, no survivor**. The two timeouts replace `TraversalIter::next` with `Some(0)` and
`Some(1)`, an iterator that never ends; the walks and collections over it run past the test
timeout, which `cargo mutants` counts as a kill.
