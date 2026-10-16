<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 6: materialize and the maintained views

Change `ultragraph-live-state-maintained-views`, tasks 6.1 to 6.7, run on 2026-10-16. The five
phases of `openspec/specs/unified-math-tdd-protocol/spec.md`.

## Decisions settled at phase 1

- The container's third type parameter is `P: Projection<N, W> = NoProjection`; `NoProjection`
  is a zero-sized projection passing everything, so `UltraGraph<T>` and `UltraGraphWeighted<T,
  W>` keep their meaning and every caller naming `UltraGraphContainer<N, W>` compiles unchanged
  (the workspace names it only inside the crate's tests and benches). The container's `Debug`
  output gains the two new fields.
- A maintained view is stored as `ViewData<W, P>`: the projection, the mode, a mask of passing
  nodes, the passing edges as a forward and a backward arena in the Live layout (design D4), and
  the node and edge counts. `MaintainedView<'a, N, W, P>` is the public read handle, borrowing
  the graph (payloads, root, state) and the data; it implements `GraphView`, `GraphTraversal`
  and every algorithm trait through the adjacency trait, as the projection view does.
- Upkeep follows the Live state's mask rule: a node that leaves the view (removed, or re-tested
  out by `update_node`) empties its own rows and leaves its entries in its neighbours' rows
  behind the mask; a node re-entering rebuilds its own rows from the graph and refills the
  sides its neighbours emptied. `remove_edge` and `remove_edge_keyed` take entries out of both
  rows whether or not the ends pass, so a return never resurrects a removed edge. The edge count
  is kept exact: a put counts once unless both rows already held the edge; a removal counts only
  when both ends pass.
- `materialize` builds the copy through `DynamicGraph::from_parts` and `freeze`, and the
  container carries a `ParallelEdges` mode (design D8): `Merge` from `new` and `with_capacity`,
  the view's from `materialize`, kept by every transition and `freeze_clone`. A direct algorithm
  call under `Distinct` runs on the simple projection with that mode, through a crate-private
  `distinct_view`; under `Merge` the states' own implementations run as before.
- `GraphState` gained crate-internal reads (`node_slots`, `get_node`, `edges_of`, `sources_of`)
  the upkeep makes of whichever state the graph is in; `CsmGraph::into_parts` and the arena's
  methods are shared with the Live state.

## Corner cases A–K

| # | Class | Instance | Test |
|---|---|---|---|
| A | Empty | A view maintained on an empty graph, then nodes arriving; a view passing nothing materialised; algorithms on an empty view | `test_a_view_on_an_empty_graph` (`graph_maintain_tests.rs`); `test_materialising_a_view_that_passes_nothing` (`graph_project_tests.rs`) |
| B | Single element | A view passing one node, with a self-loop in scope and an edge out of it | `test_a_view_passing_one_node` |
| C | Coinciding quantities | A view passing everything, equal to the graph, beside one that differs; `Merge` and `Distinct` views whose betweenness coincides beside bridges where they differ | `test_a_view_passing_everything_equals_the_graph`; `test_the_views_mode_governs_betweenness`, `test_distinct_keeps_a_parallel_pair_from_being_a_bridge` (`maintained_view/graph_algo_tests.rs`) |
| D | Degenerate index | A node leaving and returning, its entries kept behind the mask in its neighbours' rows; the view rebuilt at new positions after `compact_ids` | `test_a_node_leaving_the_scope_leaves_with_its_edges`, `test_compact_ids_carries_the_view_through_the_renumbering` |
| E | Thresholds, both sides | n/a: the views have no threshold; the rows' capacity thresholds are the Live state's, tested in group 4 | — |
| F | Zero | Zero maintained views, every test of the earlier groups; a weight of 0 in a view, routed | the suites of groups 1 to 5; `test_a_zero_weight_in_a_view` |
| G | Negative | n/a | — |
| H | Exact boundary | The test flipping on the last node, the last position leaving and returning; the replacement whose payload crosses the edge test | `test_the_view_test_flips_on_the_last_node`, `test_a_view_follows_a_payload_replacement` |
| I | Non-finite | n/a: payloads are never compared by the storage | — |
| J | Overflow reach | n/a | — |
| K | Every payload kind | A keyed, ordered, addable payload (`Weighted`) with a caller's enum projection through the four-view drive; `()` and `u64` with function-pointer projections in the view's read, traversal and identifier tests; a kind-keyed enum payload in the algorithm tests | `test_four_views_agree_with_fresh_projections_over_generated_sequences`; `maintained_view/*`, `view_id_tests.rs` |

**Independent sources.** A maintained view's expected content is a projection built afresh after
every step: `freeze_clone` of the graph, then `project` with the same tests and mode, a view that
filters at read time over the CSR and shares no code with the eager maintenance. A materialised
copy's expected content is the view it came from, over random frozen graphs, tests and both
modes. Hand-derived fixtures carry the rest.

## Phase 1 (task 6.1)

`NoProjection`, `ViewId`, `MaintainedView` with its five impls, `ViewData` with every body
`unimplemented!()`, the container's third parameter with its default, `views` and `mode` fields,
`maintain`, `view`, `mode`, `freeze_clone` returning `UltraGraphContainer<N, W>` without views,
`ProjectionView::materialize` unimplemented; every container impl generic over `P`. The
workspace builds; the 312 tests of group 5 pass on the surface but the pinned `Debug` string,
updated for the two new fields.

## Phase 2 (task 6.3)

Thirty-eight tests: `graph_maintain_tests.rs` (every scenario of `ultragraph-maintained-views`
and the three view scenarios of `ultragraph-three-states`, rows A to H, the four-view drive over
three seeds and 250 steps each), `maintained_view/{graph_view,graph_traversal,graph_algo}_tests.rs`,
`no_projection_tests.rs`, `view_id_tests.rs`, and six materialisation tests in
`graph_project_tests.rs` (the scenarios of `ultragraph-materialized-projection` and the delta on
`ultragraph-projection-view`). Bazel suites `maintained_view_tests` and `types_tests` added.

Exit run: **36 failed, 314 passed** of 350, every failure the unimplemented panic in
`maintained_view/data.rs` or `projection_view/mod.rs`.

The first run of the throwaway found four defects in the suite and one in the code. The suite:
an odd-payload fixture miscounted, a Kahn order and a path length (which counts nodes) written
wrong, and the kind an `Add` of a zero weight keeps. The code: a node re-entering a view found
its forward entry still in a neighbour's row and skipped the backward insert and the count; the
`put` was rewritten to insert each side where it is missing and count once unless both held it.

## Phase 3 (task 6.4)

A throwaway implementation, 19 defects by `audit.py` (scratchpad), the whole suite against each:

| Defect | Class | Tests failing |
|---|---|---:|
| V1 a view not updated on an edge mutation | named | 7 |
| V2 a view not updated on a node payload change | named | 5 |
| V3 a view not updated on `clear` | named | 1 |
| V4 a view updated on a mutation outside its scope | named | 4 |
| V7 a materialised copy answering under `Merge` | named | 3 |
| V8 a transition dropping the views | named | 9 |
| V9 `compact_ids` leaving a view on the old positions | named | 2 |
| G1 the mask one position short | off-by-one | 29 |
| G2 a node that fails dropped again | flipped comparison | 5 |
| G3 a passing node counted twice | constant factor | 3 |
| G4 a pair removal leaves the backward run | early return | 1, the drive |
| G5 an edge to a failing node reads as present | guard removed | 1, the drive |
| G6 the root read unfiltered | plausible neighbour | 2 |
| G7 a keyed removal of a stale entry counted | guard removed | 1, the drive |
| G8 every put counts, a replacement included | guard removed | 1, the drive |
| G9 the graph's mode ignored in the Live state | early return | 1 |
| G10 rows pushed for passing nodes only | guard removed | 1, the drive |
| G11 the node count reads the table | plausible neighbour | 9 |
| G12 a node leaving keeps its edges counted | early return | 3 |

Two named defects cannot be written against the API: `freeze_clone` copying the views (its
return type carries none) and `materialize` sharing storage with the source (the copy owns a
`CsmGraph` built from cloned parts); both are excluded by construction. No survivor. Tolerance:
none. Input variety: row C. The throwaway was discarded (the phase-1 files restored, the 36
failures observed again) before phase 4.

## Phase 4 (task 6.5)

The audited throwaway re-applied. `cargo test -p ultragraph`: 350 passed; clippy clean, no
`allow`. Coverage: every line of `maintained_view/`, `no_projection.rs`, `view_id.rs`, the
container's hooks and `materialize` is reached by a named test above; no coverage tool was run.

## Phase 5 (task 6.6)

`cargo mutants -p ultragraph --in-diff` over every source line the group added or changed: **90
mutants, 84 caught, 3 unviable, 3 missed**. The three survivors sat in one expression, the
Dynamic arm of `GraphState::node_slots`, which took the larger of one past the last live index
and the node table's length; the table's length is never smaller, so the first operand was dead
and its arithmetic mutants changed nothing. The arm now reads the table's length alone, and the
four mutants of `node_slots` and `table_len` are caught on re-run.
