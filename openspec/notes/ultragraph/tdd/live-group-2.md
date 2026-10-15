<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 2: the two challengers behind the storage trait

Change `ultragraph-live-state-maintained-views`, tasks 2.1 to 2.14. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran. Part A is the
snapshot-log candidate (design D5), run on 2026-10-16; part B the indexed lists (design D5b).

## Part A: the snapshot-log arrays

### The storage trait (task 2.1)

`src/types/storage/edge_list/mod.rs`: the crate-internal `EdgeList<V, S>` trait, one node's
edges in one direction as entries `(other, value)` with no two sharing `(other, key)`, the key
given by an `EntryKey` strategy, `ByPayload` for the forward list (the payload's `edge_key`) and
`ByValue` for the backward list (the key itself). Operations: `with_capacity`, `from_entries`,
`into_entries`, `entries`, `contains`, `contains_keyed`, `put` (true when inserted, false when
replaced), `remove_all` (the number removed), `remove_keyed`, `storage_bytes`. The type aliases
`Forward<W>` and `Backward<W>` resolve to `VecList` by default and to `EdgeArray` under the
temporary Cargo feature `edge-log`; the two modules are gated so that exactly one is compiled.

`VecList` (`edge_list/vec_list/`) is the incumbent's `Vec<(usize, V)>` behind the trait, and
`DynamicGraph` now calls the trait in `add_node`, `remove_node`, `add_edge`, `remove_edge`,
`remove_edge_keyed`, `contains_edge`, `contains_edge_keyed`, `get_edges`, both traversals,
`freeze`, `compact_ids`, `to_parts`, `construct`/`rebuild_backward` and the accounting. The
refactor changed no behaviour: the 234 tests of group 1 pass unchanged, and
`cargo mutants` over the `VecList`, the trait module and the refactored dynamic-graph lines
caught 33 of 33 (6 unviable) and 41 of 41.

### Corner cases A–K (task 2.2)

One test named per row, or `n/a` with the reason. `TRIGGER` in the tests is the candidate's
`MIN_LOG`, 16; a `freeze`/`unfreeze` round trip rebuilds every list as a snapshot with an empty
log, so a test sets a snapshot's length that way.

| # | Class | Instance | Test |
|---|---|---|---|
| A | Empty | A node with no edge: nothing listed, every removal reported, the log untouched by a failed removal | `test_a_node_without_edges_answers_empty` (`tests/types/storage/edge_list/edge_list_tests.rs`); `test_a_failed_pair_removal_writes_nothing` (`tests/utils/utils_tests_tests.rs`) |
| B | Single element | One edge through put, replacement, a second key, keyed removal, pair removal | `test_one_edge_through_every_operation` |
| C | Coinciding quantities | A snapshot exactly `MIN_LOG` long, where the two arms of the trigger coincide; beside snapshots of 5 and 40, where one arm dominates | `test_inserts_around_the_trigger_with_a_snapshot_of_the_trigger_length`, `..._short_snapshot`, `..._long_snapshot` |
| D | Degenerate index | A removed node whose entries still sit in its neighbours' snapshots; the first entry of a snapshot run (the off-by-one of the partition point) | `test_a_removed_node_leaves_the_snapshots_of_its_neighbours`; `test_parts_given_out_of_order_are_looked_up_correctly` |
| E | Thresholds, both sides | 15, 16 and 17 writes against a short snapshot and 39, 40, 41 against a long one, the contents checked; the buffers' growth at 15, 16 and 17 writes, which pins where the compaction runs | the three `test_inserts_around_the_trigger_*` tests; `test_the_buffers_grow_where_the_candidate_grows_them` |
| F | Zero | A weight of 0, stored, read back and counted before and after a compaction; a graph of no edge | `test_a_zero_weight_is_an_edge`; row A |
| G | Negative | n/a: positions, keys and counts are unsigned or ordered only | — |
| H | Exact boundary | A second and a third crossing of the trigger; a replacement, a keyed removal and a pair removal as the 16th write, each sizing the new snapshot by the count after it | `test_inserts_across_two_triggers`; `test_a_replacement_does_not_grow_the_snapshot`, `test_a_keyed_removal_shrinks_the_snapshot_by_one`, `test_a_pair_removal_shrinks_the_snapshot_by_every_key` |
| I | Non-finite | n/a: float payloads have `Key = ()` and are never compared by the storage | — |
| J | Overflow reach | n/a: the count follows the entries, bounded by memory | — |
| K | Every payload kind | `()`, `u64`, an identity key `Rel` and a field key `Tagged`, each driven through the model | the four `test_model_based_keyed_edges_with_*` tests |

The model: `BTreeMap<(source, target, key), payload>` beside the graph, compared after every
step through `number_edges`, `contains_node`, `get_edges`, both traversals, `contains_edge` and
`contains_edge_keyed` for every pair and key; 1,500 random steps on five nodes of keyed adds,
keyed and pair removals, a node removal followed by a fresh node, and `freeze`/`unfreeze` round
trips, which turn every log into a snapshot. The order the log must keep: a put after a removal
of the same key, read from the log and after a compaction; a pair removal followed by a put; and
repeated puts of one edge compacting to one.

### Phase 1 (task 2.1)

`EdgeArray<V, S> { snapshot, log, count }`, `LogEntry { other, op }`, `LogOp::{Put, Remove,
RemoveAll}` and `MIN_LOG = 16` in `edge_list/edge_array/mod.rs`; `Clone`, `Default` and `Debug`
in their files; the `EdgeList` impl in `edge_array/edge_list.rs` with every body
`unimplemented!()` (the `entries` body a `from_fn` that panics on its first `next`). `cargo build
-p ultragraph --features edge-log` passes.

### Phase 2 (task 2.3)

Eighteen tests in `tests/types/storage/edge_list/edge_list_tests.rs` (registered through
`tests/types/storage/`, the Bazel suite `storage_tests`), and the accounting tests of group 1
rewritten with one literal per candidate (`HEADER`, `UNIT`, `WEIGHT`, `BYTE`, `TAGGED`, each
derived in its comment), plus the five trigger and count tests named in rows A, E and H, so the
two configurations execute the same count.

Exit run under `--features edge-log`: **208 failed, 46 passed** of 254; every failure a panic
in `edge_array/edge_list.rs`, the unimplemented bodies; the 46 passes touch no edge list (empty
graphs, errors, the frozen state built from no edge, the trait tests).

### Phase 3 (task 2.4)

A throwaway implementation, 17 defects injected one at a time by `audit.py` (scratchpad), the
whole suite run under `--features edge-log` against each:

| Defect | Class | Tests failing |
|---|---|---:|
| N1 a duplicate insert kept by an all-insert compaction | named | 20 |
| N3a compaction one entry early | named | 1, the growth test |
| N3b compaction one entry late | named | 1, the growth test |
| N4 an older removal suppresses a newer put | named | 10 |
| N5 snapshot entries to a removed node kept | named | 12 |
| G1 off-by-one: a snapshot run starts after its first entry | off-by-one | 20 |
| G2 the keyed binary search compares the wrong way | flipped comparison | 6 |
| G3 the trigger halved | constant factor | 1, the growth test |
| G4 `contains` reads the log's mentions only | early return | 15 |
| G5 every put counts, a replacement included | guard removed | **0, then 1** |
| G6 the keyed test ignores the key | plausible neighbour | 6 |
| G7 `from_entries` leaves the snapshot unsorted | guard removed | **0, then 1** |
| G8 `into_entries` skips the log | early return | 112 |
| G9 `remove_keyed` removes without looking | guard removed | 8 |
| G10 storage counted by length | plausible neighbour | 10 |
| G11 a removal of every key ignored by compaction | guard removed | 21 |
| G12 a read with a log ignores the snapshot | early return | 13 |

N2, a zero weight read as a removal, cannot be written: a removal is a `LogOp` variant and the
storage never compares a payload with anything; `test_a_zero_weight_is_an_edge` stands as the
guard on the contract. Two survivors widened the suite: G5, because the count only sizes the
snapshot a compaction builds, is now pinned by `test_a_replacement_does_not_grow_the_snapshot`
through the byte count; G7, because every other path hands `from_entries` sorted input, is now
pinned by `test_parts_given_out_of_order_are_looked_up_correctly`. Both re-injected: caught.
The throwaway was discarded (the phase-1 file restored, 208 failures observed again) before
phase 4. Tolerance: none. Input variety: row C pairs the coinciding trigger with both
dominating arms; row K pairs the unit key with keys that differ from their payload.

Two defects the throwaway itself had, found by the suite before the audit: a compaction run
before the count moved, which reserved the new snapshot one entry short and doubled it (the
growth test read 3,536 for 3,424), and a resolved read with no `size_hint`, which the container's
size-hint test caught. Both fixed in the throwaway and carried into phase 4.

### Phase 4 (task 2.5)

The implementation is the audited throwaway re-applied. `cargo test -p ultragraph`: 257 passed
without a feature and 257 with `--features edge-log`; clippy clean under both; `cargo bench -p
ultragraph --features edge-log -- --test` passes every storage group. Coverage: every line of
`edge_array/` is reached by a named test above; no coverage tool was run.

### Phase 5 (task 2.6)

`cargo mutants -p ultragraph --features edge-log` over `edge_array/*.rs`: 89 mutants, 73 caught,
11 unviable (returns of leaked or defaulted values the types refuse), **5 missed**: the `> 0`
guard of `remove_all` and the count arithmetic of `remove_all` and `remove_keyed`, every one a
count that only a compaction's capacity exposes. Closed by `test_a_failed_pair_removal_writes_nothing`,
`test_a_keyed_removal_shrinks_the_snapshot_by_one` and
`test_a_pair_removal_shrinks_the_snapshot_by_every_key`, each deriving the byte count the
mutant would produce beside the right one; re-run over the two functions: 12 of 12 caught.

### Measurements and close-out of part A

The incumbent re-measured behind the trait, three runs, recorded as "Group 2" in
`BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`: the hub rows within 1 % of group 1,
the three low-degree keyed removals 9 to 17 % slower (a one-entry list paying a position scan
and the trait's key extraction), the rest within tolerance. Group 3 compares the candidates
with these rows, the same build.

`cargo bench -p ultragraph --features edge-log -- --test` runs every storage group on the
snapshot-log arrays and prints the hub at **64.973 bytes per edge** at `MIN_LOG = 16`,
6,497,264 bytes over 100,000 edges, against the incumbent's 48.972: above the 1.25× gate of
61.2 bytes per edge at this `MIN_LOG`. The derivation of that count and the sweep of `MIN_LOG`
belong to task 3.1, which decides whether any value passes the gate.

`bazel test //...`: 1,530 pass (the `storage_tests` suite among them); clippy clean under both
features; `openspec validate --all`: 235 pass.

## Part B: the indexed lists

Run on 2026-10-16, design D5b, tasks 2.8 to 2.14.

### Corner cases A–K (task 2.9)

| # | Class | Instance | Test |
|---|---|---|---|
| A | Empty | A node with no edge, no index; the parts of an empty hub | row A of part A |
| B | Single element | One edge through every operation, far below the threshold | `test_one_edge_through_every_operation` |
| C | Coinciding quantities | Entries equal to targets (one key each, the accounting test) beside targets carrying two and three keys (the threshold test, the hub model), where the index's outer and inner maps differ in size | `test_the_index_is_counted_from_the_threshold_until_below_half_of_it`; `test_a_hub_across_the_threshold_in_both_directions` |
| D | Degenerate index | The entry a swap-remove moves into the removed position, re-recorded; a target node removed from an indexed hub and the positions compacted | `test_model_based_hub_across_the_index_threshold`; `test_an_indexed_hub_survives_a_removal_and_a_compaction` |
| E | Thresholds, both sides | 63, 64 and 65 entries growing; 33, 32 and 31 shrinking; the index counted from 64 until below 32; a second crossing upward | `test_a_hub_across_the_threshold_in_both_directions`; `test_the_index_is_counted_from_the_threshold_until_below_half_of_it` |
| F | Zero | A weight of 0 (part A); a hub shrunk to three entries after its index left | `test_a_zero_weight_is_an_edge`; the threshold test |
| G | Negative | n/a: positions, keys and degrees are unsigned or ordered only | — |
| H | Exact boundary | A list of exactly `HUB_DEGREE` entries handed to `from_parts` and to `unfreeze`, the index built from entries; a pair removal taking two keys of one target on an indexed hub | `test_parts_and_unfreeze_rebuild_a_hub_above_the_threshold`; the threshold test |
| I | Non-finite | n/a: as part A | — |
| J | Overflow reach | n/a: positions follow the list | — |
| K | Every payload kind | The four kinds of part A through the model; the hub tests on the field-keyed `Tagged` and the accounting on `()` | part A's model tests; the hub tests |

The hub model: node 0 against 40 targets and three kinds, 2,500 steps in three phases (growing,
shrinking, growing), with `freeze`/`unfreeze` round trips and `compact_ids` with no tombstone,
both of which rebuild the hub's list from entries; the run asserts that the degree crossed
`HUB_DEGREE` upward in each growing phase and fell below half of it in between.

### Phase 1 (task 2.8)

`IndexedList<V, S> { entries, index: Option<BTreeMap<usize, BTreeMap<Key, usize>>> }`,
`HUB_DEGREE = 64`, `BTREE_NODE_HEADER = 16`, `BTREE_NODE_ENTRIES = 11` in
`edge_list/indexed_list/mod.rs`; `Clone`, `Default`, `Debug` in their files; the `EdgeList` impl
with every body `unimplemented!()`. The feature `edge-index`; `edge-log` and `edge-index`
together hit the `compile_error!` of `edge_list/mod.rs`. Every configuration builds.

### Phase 2 (task 2.10)

Five hub tests in `edge_list_tests.rs` and the accounting test of row E; the accounting
constants made three-way, the indexed list's header being a `Vec` and an `Option<BTreeMap>`,
56 bytes (the option carries its own tag beside the map's 24). Two test defects found by the
incumbent before the candidate ran: the shrink helper destructured the model key in the wrong
order, and the first random drive settled near 70 edges and never fell below half, hence the
three phases.

Exit run under `--features edge-index`: **216 failed, 46 passed** of 262, every failure the
unimplemented panic in `indexed_list/edge_list.rs`, but one: the accounting test with no node,
which read a 56-byte header where the test said 48, corrected before the audit.

### Phase 3 (task 2.11)

A throwaway implementation, 15 defects by `audit_index.py` (scratchpad), the whole suite under
`--features edge-index` against each:

| Defect | Class | Tests failing |
|---|---|---:|
| I1 the moved entry not re-recorded after a swap-remove | named | 3 |
| I2 the index built one entry below the threshold | named | 1, the accounting test |
| I3 the index built one entry above the threshold | named | 1, the accounting test |
| I4 the index never dropped | named | 1, the accounting test |
| I4b the index dropped at half rather than below it | named | 1, the accounting test |
| I5 an empty index after `from_entries` | named (stale index after `compact_ids`, `unfreeze`) | 3 |
| I6 a new entry not recorded in the index | named (index and list disagree) | 5 |
| G1 positions removed lowest first | flipped comparison | 1, the hub model |
| G2 node headers doubled in the accounting | constant factor | 1 |
| G3 an unindexed pair removal reports the length | early return | 11 |
| G4 an emptied inner map kept | guard removed | 3 |
| G5 the accounting ignores the index | plausible neighbour | 1 |
| G6 the indexed lookup ignores the key | plausible neighbour | 6 |
| G7 the inner index records the position after the entry | off-by-one | 4 |
| G8 the indexed pair test reads the list's first entry | plausible neighbour | 2 |

No survivor. Tolerance: none. Input variety: row C. The throwaway was discarded (216 failures
observed again on the stub) before phase 4.

### Phase 4 (task 2.12)

The audited throwaway re-applied. `cargo test -p ultragraph`: 262 passed without a feature, 262
with `--features edge-log` and 262 with `--features edge-index`; clippy clean under all three
(three test lints fixed, no `allow`); `cargo bench -p ultragraph --features edge-index -- --test`
passes every storage group; `bazel test //...`: 1,530 pass. Coverage: every line of
`indexed_list/` is reached by a named test above; no coverage tool was run.

### Phase 5 (task 2.13)

`cargo mutants -p ultragraph --features edge-index` over `indexed_list/*.rs`: 68 mutants, 59
caught, 9 unviable, no survivor.

### Observations for group 3

The bench self-test prints the candidates' memory at their default parameters, against the
incumbent's 48.972 bytes per edge on the hub and 176.048 on the chain of 1,000:

| Candidate | Hub | Chain of 1,000 |
|---|---:|---:|
| snapshot-log arrays, `MIN_LOG = 16` | 64.973 | 240.112 |
| indexed lists, `HUB_DEGREE = 64` | 96.336 | 240.112 |

Both headers are 56 bytes against the `Vec` list's 24, which alone adds 32 bytes per node and
direction: on the chain, 64,064 bytes over 1,000 edges, the whole of the chain's rise. The 1.25×
gate is 61.2 on the hub and 220.1 on the chain; neither candidate passes it at its default, and
the chain figure is the header, which no parameter changes. The sweep of task 3.1 decides.
