<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 3: the edge-storage decision

Change `ultragraph-live-state-maintained-views`, tasks 3.1 to 3.4, run on 2026-10-16. This
group is a measurement and a decision; it adds no library code, so the five phases do not apply.
The rows and the verdict are in `BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`
(group 3) and the outcome in `openspec/notes/ultragraph/RadixGraphUpdate.md`, §Design 4 and
§Benchmark.

## The sweep (task 3.1)

`sweep.py` (scratchpad) edited each challenger's constant, built it under its feature and ran
the `memory_per_edge` group once per value: `MIN_LOG` over 4, 8, 16, 32, 64, 128 and 1,024;
`HUB_DEGREE` over 8, 16, 32, 64, 128, 256, 1,024 and 8,192. Memory per edge did not move with
the parameter in any measured configuration, and both challengers sit above the gate in all of
them, so no value qualifies and no three-run timing at "its value" exists. Both were timed at
their defaults for the record (three runs of the arrays, one of the index, the two further index
runs stopped once the decision stood and their truncated log removed); those rows and what they
would have meant under the rule are in the results document.

## The decision (task 3.2)

By the rule of `ultragraph-edge-storage-benchmark`: a challenger with no parameter value at or
under 1.25× the `Vec` lists' memory per edge in every measured configuration is out; none
qualifies; the `Vec` lists stay. Recorded in `RadixGraphUpdate.md` §Benchmark with the gate rows,
the swept values and every candidate's memory per edge.

## The removal (task 3.3)

Approved by Marvin on 2026-10-16. Removed: `ultragraph/src/types/storage/edge_list/` (the
`EdgeList` trait, `EntryKey`, `VecList`, `EdgeArray`, `IndexedList`), the features `edge-log` and
`edge-index`, and the five tests that pinned the log's compaction trigger. The trait does not
serve the Live state, whose rows sit in one arena behind the adjacency trait of change 1 (design
D1 and D3), so it went with the candidates and `graph_dynamic/` is the code of group 1 again,
restored from that commit. Kept: the keyed-edge contract tests, moved to
`tests/types/storage/graph_dynamic/graph_mut_tests.rs` with the candidate names taken out of
their wording; the accounting tests with the `Vec` list's literals; the storage benchmark
workloads, which the later groups measure.

`cargo test -p ultragraph`: 257 pass; clippy clean; `cargo bench -p ultragraph -- --test` passes
every storage group; `bazel test //...` green; the crate has no `cfg(feature = "edge-…")` left.
