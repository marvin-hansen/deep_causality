<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 5: the Live decision

Change `ultragraph-live-state-maintained-views`, tasks 5.1 to 5.4, run on 2026-10-16. A
measurement and a decision; the group adds benchmark code only, verified by
`cargo bench -p ultragraph -- --test` and by its three runs.

## The workloads (task 5.1)

`ultragraph/benches/benchmarks/bench_live.rs`, registered in `benchmarks/mod.rs` and
`bench_main.rs`:

| Group | Rows | Fixture | What one iteration does |
|---|---|---|---|
| `live_writes_then_scc` | `live/chain_1000/{1,16,256}`, `dynamic/chain_1000/{1,16,256}`, `live/hub/{1,16,256}`, `dynamic/hub/{1,16,256}` | the keyed chain of 1,000; the keyed hub of 100,000 edges | k keyed writes (`add_edge` on an edge that exists, cycling through random edges) then `strongly_connected_components`; the `dynamic` rows add `freeze` before and `unfreeze` after the algorithm |
| `live_algorithms` | `{scc,topological_sort,shortest_path_len}/{frozen,live}` | the chain of 1,000, unchanged | one algorithm; the Live rows read the rows through the mask |
| `live_update` | `in_place`, `rebuild` | a clone of the Live chain per batch | a keyed insert into a row with a spare slot; into the full row of capacity one every chain node starts with |
| `live_transitions` | `make_live/{from_dynamic,from_frozen}`, `freeze/from_live`, `unfreeze/from_live`, `freeze_clone/{dynamic,frozen,live}` | the chain of 1,000, cloned per batch where consumed | one transition |

A keyed write is a replacement, so the structure stays and a row can iterate on one graph;
the writes are the same random edges on both paths. The `dynamic` rows are the workflow Live
replaces: writes in the Dynamic state and a freeze before each analysis, the graph unfrozen
again for the next writes.

`cargo bench -p ultragraph -- --test '^live_'`: 27 rows pass.

## The rule (task 5.2)

`ultragraph-edge-storage-benchmark`, "The Live state stays only if it earns its place": for
k = 1, 16 and 256, at 1,000 nodes and on the hub, Live is *clearly faster* than Dynamic →
`freeze` → the algorithm → `unfreeze`: each of its three runs faster than each of the other's,
and its mean below the other's by more than the other row's spread times its mean. Six rows,
every one has to hold.

## The decision (task 5.2)

Three runs (`BenchAfterChange2LiveView/Group5-LiveRun-1.txt` to `-3.txt`), aggregated and
judged by `verdict.py` (scratchpad); the tables are in
`BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`, group 5. Live against the
freeze-per-analysis path, means: chain of 1,000 at k = 1, 16, 256: 44.7, 44.9, 47.8 µs against
145.3, 145.0, 145.3 µs; hub: 1.04, 1.05, 1.08 ms against 7.53, 7.90, 13.17 ms. Every Live run
below every other run on all six rows, the margin a multiple of the spreads. **Live stays**;
task 5.3 does not arise.
