<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 1: benchmark workloads for the decisions

Change `ultragraph-live-state-maintained-views`, tasks 0.1 to 1.3. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran on 2026-10-15. The
library code of this group is the edge-storage accounting of design D6; the workloads are
benchmark code, verified by `cargo bench -p ultragraph -- --test` and by their three runs.

Task 0.1: `ultragraph-stable-ids-keyed-edges` is archived
(`openspec/changes/archive/2026-10-15-ultragraph-stable-ids-keyed-edges`) and its after-benchmark
is `openspec/notes/ultragraph/BenchAfterChange1StableId/BenchAfterChange1StableId.md`. The
results of this change go to `openspec/notes/ultragraph/BenchAfterChange2LiveView/`, the folder
Marvin created for them; the change's documents named `BenchStorage/` and now name this folder.

## Corner cases A–K

One test named per row, or `n/a` with the reason. Every byte count is derived by hand from the
capacities the constructors reserve: a `Vec` header is 24 bytes, `usize` 8, `with_capacity(n,
Some(k))` reserves the two outer vectors for `n` entries and each node's two lists for `k`.

| # | Class | Instance in this group | Test (`ultragraph/tests/utils/utils_tests_tests.rs`) |
|---|---|---|---|
| A | Empty | `new()`: no buffer; `with_capacity(5, Some(3))` without a node: the outer vectors alone; the empty frozen graph: the two `[0]` offset sentinels | `test_an_empty_graph_holds_no_edge_storage`, `test_reserved_nodes_without_edges_count_their_outer_vectors_only`, `test_an_empty_frozen_graph_holds_its_offset_sentinels_only` |
| B | Single element | One edge in lists of capacity one | `test_one_edge_fills_its_list_exactly` |
| C | Coinciding quantities | Capacity equal to length (row B) beside capacity above length, where counting lengths gives 160 for 240 | `test_reserved_capacity_counts_where_the_length_does_not`, `test_the_outer_vectors_count_their_capacity` |
| D | Degenerate index | The last node's lists, reserved and empty, in the three-node fixture | `test_three_nodes_two_edges_count_every_reserved_list` |
| E | Thresholds, both sides | n/a: the accounting has no threshold | — |
| F | Zero | A weight of 0, counted as any weight; a graph with no edge, where the per-edge figure is `None` | `test_a_zero_weight_counts_as_any_weight`, the row-A tests |
| G | Negative | n/a: capacities and sizes are `usize` | — |
| H | Exact boundary | A list filled to its capacity exactly | `test_one_edge_fills_its_list_exactly` |
| I | Non-finite | n/a: no arithmetic on payloads; the per-edge figure divides a byte count by an edge count ≥ 1 | — |
| J | Overflow reach | n/a: byte counts of allocated buffers fit `usize` by construction | — |
| K | Every payload kind | `()`; a weight (`u64`, forward 16 bytes, backward 8); an identity key (`Rel`, 16 both ways); a field key (`Tagged`, keyed by a `u8`); each in the dynamic state, `()` and `u64` frozen as well | `test_a_weight_counts_its_size_forward_and_its_unit_key_backward`, `test_an_identity_key_counts_its_size_in_both_directions`, `test_a_field_key_counts_the_payload_forward_and_the_key_backward`, `test_a_frozen_graph_counts_its_csr_arrays`, `test_a_frozen_weighted_graph_counts_its_weights` |

The frozen rows pin the CSR as `freeze` builds it: offsets reserved for `slots + 1`, targets and
weights of exactly `edges` entries, both directions.

## Phase 1 — the API, every new body `unimplemented!()`

`DynamicGraph::edge_storage_bytes` (`src/types/storage/graph_dynamic/memory.rs`),
`CsmGraph::edge_storage_bytes` (`src/types/storage/graph_csm/memory.rs`), the container's
dispatch (`src/types/ultra_graph/memory.rs`), all `pub(crate)`; and in `src/utils/utils_tests.rs`
the `#[doc(hidden)]` pair `edge_storage_bytes(&graph) -> usize` and `memory_per_edge(&graph) ->
Option<f64>`. `cargo build -p ultragraph` passes with two dead-code warnings, the two state
bodies the unimplemented dispatch does not yet call.

## Phase 2 — the suite, observed failing

Fourteen tests in `tests/utils/utils_tests_tests.rs`, registered through `tests/utils/mod.rs`
and `tests/mod.rs`, and the Bazel suite `utils_tests` in `ultragraph/BUILD.bazel`.

Exit run: **14 failed, 0 passed**, every failure `panicked at
ultragraph/src/types/ultra_graph/memory.rs:14:9`, the unimplemented dispatch, which every test
reaches first.

## Phase 3 — the defect audit

A throwaway implementation, twelve defects injected one at a time by the script
`audit.py` (scratchpad), the fourteen tests run against each, the throwaway discarded by
restoring the phase-1 files before phase 4.

| Defect | Class | Tests failing | Among them |
|---|---|---:|---|
| D1 inner lists counted by length | plausible neighbour | 9 | `test_reserved_capacity_counts_where_the_length_does_not` |
| D2 outer vectors counted by length | plausible neighbour | 2 | `test_the_outer_vectors_count_their_capacity`, the reserved-without-nodes test |
| D3 forward lists only | early return | 10 | `test_three_nodes_two_edges_count_every_reserved_list` |
| D4 the backward entry sized as the payload | plausible neighbour | 2 | `test_a_weight_counts_its_size_forward_and_its_unit_key_backward` |
| D5 divided by the node count | plausible neighbour | 10 | every per-edge assertion |
| D6 frozen: forward CSR only | early return | 3 | the three frozen tests |
| D7 frozen: offsets dropped | guard removed | 3 | the three frozen tests |
| D8 the no-edge guard removed | guard removed | 3 | the row-A tests |
| D9 off-by-one: the last node's lists skipped | off-by-one | 9 | `test_three_nodes_two_edges_count_every_reserved_list` |
| D10 every entry sized as one `usize` | constant factor | 4 | the four payload-kind tests |
| D11 the guard flipped | flipped comparison | 13 | all but one |
| D12 list headers halved | constant factor | 10 | every dynamic test with a node |

No survivor. Tolerance: none. Input variety: row C pairs every capacity-equals-length case with
one where they differ, so a length-based count cannot pass; row K pairs the unit key with keys
of a different size from their payload, so a backward count sized by the payload cannot pass.

## Phase 4 — implementation

`edge_storage_bytes` sums, per direction, the outer vector's capacity times the list-header size
and each list's capacity times its entry size, `(usize, W)` forward and `(usize, W::Key)`
backward; the frozen state sums the offsets, targets and weights of both CSRs; the container
matches on its state; `memory_per_edge` divides by `number_edges` and is `None` at zero edges.
`cargo test -p ultragraph`: 234 passed (220 before this group); `bazel test //ultragraph/...`: 21
suites pass, the new `utils_tests` among them; clippy clean. Coverage: every added line is
reached by a named test above; no coverage tool was run.

## Phase 5 — mutation testing

`cargo mutants -p ultragraph` over the four files of phase 1: **47 mutants, 47 caught**, no
timeout, no unviable, no survivor.

## The workloads

`benches/benchmarks/bench_storage.rs`, registered in `benches/benchmarks/mod.rs` and
`benches/bench_main.rs`; the keyed payload `Kind(u8)` with `Key = u8` in `data.rs`; the fixtures
`build_keyed_chain`, `build_keyed_hub` and the shared `walk` in `utils.rs` (`walk` moved there
from `bench_crud.rs`, whose four walks call it).

| Group | Parameter | Fixture | Operation |
|---|---|---|---|
| `keyed_update_low` | 10, 100, 1,000 | keyed chain | `add_edge(i, i + 1, kind_of(i))`: replaces the payload of a random edge |
| `keyed_update_hub` | 100,000 | keyed hub, 25,000 leaves × 4 kinds | `add_edge(0, leaf, kind)`: replaces a random keyed edge of the hub |
| `keyed_remove_low` | 10, 100, 1,000 | keyed chain, fresh per batch | `remove_edge_keyed(i, i + 1, key)` |
| `keyed_remove_hub` | 100,000 | keyed hub, cloned per batch, 20 samples | `remove_edge_keyed(0, leaf, key)` |
| `outbound_walk_dynamic`, `inbound_walk_dynamic` | 10, 100, 1,000 | chain, dynamic | the walk of `bench_crud`, no freeze |
| `outbound_walk_shuffled_dynamic`, `inbound_walk_shuffled_dynamic` | 10, 100, 1,000 | shuffled chain, dynamic | the same |
| `memory_per_edge` | `hub_built`, `hub_updated`, `chain_1000_built`, `chain_1000_updated` | keyed hub; keyed chain of 1,000 | prints the byte count, then times the accounting call |

`cargo bench -p ultragraph -- --test` passes every group. The incumbent's memory per edge,
printed by the `memory_per_edge` group and derived by hand below, is recorded in
`BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`.
