<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# UltraGraph benchmark commands

The commands behind the logs in `BenchBefore/`, `BenchAfterChange1StableId/` and
`BenchAfterChange2LiveView/`, for re-running any of them. Benchmark machine: M3 Max, 16 cores,
128 GB. Criterion's defaults are a 3 s warm-up and 100 samples over 5 s per row, so a row costs
about 8 to 10 s whatever it measures; the whole crate is about 170 rows, 25 minutes per run.

## One full run

```sh
cargo bench -p ultragraph 2>&1 | sed "s#$PWD#<workspace>#g" > <folder>/Run-1.txt
```

The three-run protocol repeats this three times and aggregates with
`BenchAfterChange2LiveView/compare.py`, which reads the three logs and prints the per-row mean,
spread and the comparison with the incumbent table it is pointed at.

## One group, or a set of groups

The argument after `--` is a regular expression over the row id (`group/function/parameter`).

```sh
cargo bench -p ultragraph -- '^view_upkeep'
cargo bench -p ultragraph -- '^(live_writes_then_scc|live_algorithms|live_update|live_transitions)'
```

## A shorter row

Criterion's timing flags are accepted by the bench binary only, so name it; without `--bench`
Cargo also runs the lib test harness, which rejects them.

```sh
cargo bench -p ultragraph --bench bench_main -- '^inbound_walk_dynamic' --warm-up-time 1 --measurement-time 2
```

## A/B against an earlier commit, back to back

Builds the earlier commit from a `git archive` in a scratch directory and runs the shared groups
on both trees one after the other, so that both see the same machine state. The pair in
`BenchAfterChange2LiveView/Group7-AB-Old.txt` and `Group7-AB-New.txt` was made this way against
c397d4d31, the commit before the Live state.

```sh
A=/tmp/ab_old && mkdir -p $A && git archive c397d4d31 | tar -x -C $A
(cd $A && cargo build -p ultragraph --benches --release)

F='^(build_graph|add_node|get_node|articulation_points|bridges|biconnected_components|add_edge|contains_edge|get_edges|outbound_walk|inbound_walk|update_node|remove_node|remove_edge|freeze|unfreeze|keyed_|memory_per_edge)'
(cd $A && cargo bench -p ultragraph -- "$F") 2>&1 | sed "s#$A#<old-tree>#g" > AB-Old.txt
cargo bench -p ultragraph -- "$F" 2>&1 | sed "s#$PWD#<workspace>#g" > AB-New.txt
python3 openspec/notes/ultragraph/BenchAfterChange2LiveView/ab.py AB-Old.txt AB-New.txt
```

`ab.py` prints the row-by-row ratio and marks every move beyond 5 %. For the rows the first
pass flagged, the shorter timing above cuts a pass to about two minutes per tree:

```sh
F='^(build_graph|add_node|articulation_points|bridges|biconnected_components|keyed_update_low|keyed_remove_low|outbound_walk_dynamic|inbound_walk_dynamic|inbound_walk_shuffled_dynamic)'
(cd $A && cargo bench -p ultragraph --bench bench_main -- "$F" --warm-up-time 1 --measurement-time 2) > AB-Old.txt
cargo bench -p ultragraph --bench bench_main -- "$F" --warm-up-time 1 --measurement-time 2 > AB-New.txt
```

## Mutation testing of a file

```sh
cargo mutants -p ultragraph --file 'ultragraph/src/types/storage/graph_live/row_entries.rs' -j 8
```
