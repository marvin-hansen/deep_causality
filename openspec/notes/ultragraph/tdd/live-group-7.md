<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Live group 7: measurement and close-out

Change `ultragraph-live-state-maintained-views`, tasks 7.1 to 7.4, run on 2026-10-16. The group
adds the last workload and measures the whole crate; no library code changes.

## The view-upkeep workload (task 7.1)

`benches/benchmarks/bench_views.rs`, group `view_upkeep`: a keyed write (`add_edge` on an edge
that exists) and a payload update that keeps the node in scope, each on the keyed chain of 1,000
nodes with 0, 1, 4 and 16 maintained views registered, in the Dynamic and the Live state. Every
view passes every node and edge, so each write updates every view's rows: the most a view can
cost. `cargo bench -p ultragraph -- --test '^view_upkeep'`: 16 rows pass.

## The final measurement (task 7.1)

Three runs of the whole `cargo bench -p ultragraph`, every group, the logs
`BenchAfterChange2LiveView/Group7-FinalRun-1.txt` to `-3.txt`, aggregated by `compare.py`
(scratchpad) and compared row by row with the incumbent each row has: the after-benchmark of
change 1 for the rows that existed then, the group-2 incumbent for the storage rows, group 5 for
the Live rows; the view rows are new. The tables and every move beyond a row's spread are in
`BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`, group 7.

## The back-to-back A/B

The comparison with the incumbents mixes two things: what the code changed and what the machine
did between the incumbent's run and this one. The Live rows of the final runs read 30 % below
group 5's with no change to the code they measure, so the incumbent comparison alone cannot
attribute a move. An A/B settles it: the commit before the Live state (c397d4d31), built from a
`git archive` in a scratch directory, run directly before the current tree on the shared groups,
`Group7-AB-Old.txt` and `Group7-AB-New.txt`. Of 84 shared rows, 30 moved beyond 5 %; the code
effects among them:

- `inbound_walk_dynamic/100` +258 %, `/1000` +396 %, `outbound_walk_dynamic/1000` +60 %. The
  walk constructs one traversal iterator per step. The container's iterator is an enum over the
  three states, and the Live variant held the zip, filter and map chain of a Live row read,
  which set the enum's size for every state.
- `add_node` +16 to +42 %, `build_graph` +21 to +28 %, `keyed_update_low` +43 to +54 %. The
  view hooks ran on every graph: `add_node` read the node back and iterated an empty list,
  `add_edge` tested for views and built an `Option` around the clone.
- `articulation_points/10` +22 %, `bridges/10` to `/1000` +6 to +15 %, `biconnected_components`
  +7 %, `keyed_remove_low` +9 to +11 %: without an explanation in the diff; the dynamic graph
  and the frozen algorithms are unchanged between the trees.

## Two fixes

`src/types/storage/graph_live/row_entries.rs`: `RowEntries`, one row of an arena read through
a mask, two slices and the mask closure. `LiveGraph::live_entries` and `ViewData::passing_entries`
return it. `src/types/ultra_graph/graph_mut.rs`: `views_add_node` returns before reading the node
when the graph maintains no view, called from `add_node` and `add_root_node`; `add_edge` takes
the state's write and returns when there is no view, and clones the payload only after that test.
Suite 350 passed, clippy clean. `cargo mutants` over the two files: 19 mutants, 18 caught, 1
unviable, none missed.

## Not measured

The two fixes are not re-measured. A second A/B pass was started and stopped on Marvin's
instruction after 17 minutes of its 30; a shortened re-run over the flagged rows failed before
measuring (criterion's timing flags reached the lib test harness) and was not repeated. The
commands for either, with the fix for the flags, are in `../BenchCommands.md`. Until the pass is
run, the first A/B's figures are the record, and the two fixes are reasoned, not measured.
