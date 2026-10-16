rustc -V
rustc 1.99.0 (b940084d7 2026-09-28)

sysctl machdep.cpu.brand_string hw.ncpu hw.memsize
machdep.cpu.brand_string: Apple M3 Max
hw.ncpu: 16
hw.memsize: 137438953472 (128GB)

uname -a
27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:42 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6031 arm64

-----------------------------------------------------------------
-----------------------------------------------------------------

## Change 2: the measurements the decisions rest on

Every timing is three runs of `cargo bench -p ultragraph` on this machine, aggregated as in
`../BenchBefore/BenchBefore.md`: each run value is criterion's point estimate, the mean is over the
three runs, the spread is (largest − smallest) / mean, and a row whose spread exceeds 15 % gives
the median instead. Memory per edge is deterministic and comes from one run: the sum of the
capacity of every buffer the edge storage holds, times the element size, over the number of edges,
computed by `ultragraph::utils::utils_tests::edge_storage_bytes` and printed by the
`memory_per_edge` group into the run log.

### Group 1: the incumbent on the keyed workloads, dynamic traversal and memory

The `Vec` lists of change 1 at the end of its review follow-ups, measured on 2026-10-15; the logs
are `Group1-IncumbentRun-1.txt` to `Group1-IncumbentRun-3.txt`, the groups filtered to those of
`benches/benchmarks/bench_storage.rs`. Fixtures: the keyed chain (`n + 1` nodes, edge `i → i + 1`
of kind `i mod 4`, degree at most 1); the keyed hub (a hub with four kinds of edge to each of
25,000 leaves, 100,000 edges out of one node); the chain and the shuffled chain of `bench_crud`,
left dynamic. An update replaces the payload of an edge that exists; a removal takes one keyed
edge out, from a fresh chain or a clone of the hub per batch. Per element: per step walked.

| Benchmark | n | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---:|---:|---:|---:|---:|---:|
| `keyed_update_low` | 10 | 2.59 ns | 2.57 ns | 2.62 ns | 2.60 ns | 1.7 % |  |
| `keyed_update_low` | 100 | 2.64 ns | 2.58 ns | 2.58 ns | 2.60 ns | 2.5 % |  |
| `keyed_update_low` | 1,000 | 2.74 ns | 2.71 ns | 2.75 ns | 2.74 ns | 1.5 % |  |
| `keyed_update_hub` | 100,000 | 16.48 µs | 16.51 µs | 16.59 µs | 16.52 µs | 0.7 % |  |
| `keyed_remove_low` | 10 | 6.48 ns | 6.64 ns | 6.60 ns | 6.58 ns | 2.4 % |  |
| `keyed_remove_low` | 100 | 6.31 ns | 6.25 ns | 6.16 ns | 6.24 ns | 2.4 % |  |
| `keyed_remove_low` | 1,000 | 12.28 ns | 13.23 ns | 12.82 ns | 12.78 ns | 7.4 % |  |
| `keyed_remove_hub` | 100,000 | 30.01 µs | 29.66 µs | 29.30 µs | 29.66 µs | 2.4 % |  |
| `outbound_walk_dynamic` | 10 | 15.04 ns | 14.46 ns | 14.85 ns | 14.78 ns | 3.9 % | 1.48 ns |
| `outbound_walk_dynamic` | 100 | 278.75 ns | 291.45 ns | 280.60 ns | 283.60 ns | 4.5 % | 2.84 ns |
| `outbound_walk_dynamic` | 1,000 | 629.51 ns | 614.41 ns | 630.56 ns | 624.83 ns | 2.6 % | 0.62 ns |
| `inbound_walk_dynamic` | 10 | 14.92 ns | 14.81 ns | 14.81 ns | 14.85 ns | 0.8 % | 1.48 ns |
| `inbound_walk_dynamic` | 100 | 247.76 ns | 291.84 ns | 263.94 ns | 263.94 ns (median) | 16.5 % | 2.64 ns |
| `inbound_walk_dynamic` | 1,000 | 634.77 ns | 631.60 ns | 614.31 ns | 626.89 ns | 3.3 % | 0.63 ns |
| `outbound_walk_shuffled_dynamic` | 10 | 14.79 ns | 14.98 ns | 14.80 ns | 14.86 ns | 1.3 % | 1.49 ns |
| `outbound_walk_shuffled_dynamic` | 100 | 317.50 ns | 298.69 ns | 312.75 ns | 309.65 ns | 6.1 % | 3.10 ns |
| `outbound_walk_shuffled_dynamic` | 1,000 | 3.42 µs | 1.77 µs | 3.42 µs | 3.42 µs (median) | 57.6 % | 3.42 ns |
| `inbound_walk_shuffled_dynamic` | 10 | 14.80 ns | 14.84 ns | 14.46 ns | 14.70 ns | 2.6 % | 1.47 ns |
| `inbound_walk_shuffled_dynamic` | 100 | 282.40 ns | 226.33 ns | 311.75 ns | 282.40 ns (median) | 31.2 % | 2.82 ns |
| `inbound_walk_shuffled_dynamic` | 1,000 | 3.43 µs | 1.89 µs | 3.40 µs | 3.40 µs (median) | 52.9 % | 3.40 ns |

The `memory_per_edge` rows time the accounting call itself (10.5 µs over the hub's 25,001 lists,
218 ns over the chain's 1,001) and feed no decision; the figures that do are the byte counts:

| Fixture | Bytes | Edges | Bytes per edge | Derivation |
|---|---:|---:|---:|---|
| hub, built | 4,897,200 | 100,000 | 48.972 | outer vectors 2 × 25,001 × 24 = 1,200,048; the hub's forward list grown by pushes to capacity 131,072 × 16 bytes `(usize, Kind)` = 2,097,152; each leaf's backward list at capacity 4 × 16 bytes `(usize, u8)`, 25,000 × 64 = 1,600,000; the hub's backward and the leaves' forward lists empty, capacity 0 |
| hub, after 1,024 updates | 4,897,200 | 100,000 | 48.972 | an update replaces in place: no capacity moves |
| chain of 1,000, built | 176,048 | 1,000 | 176.048 | outer 2 × 1,001 × 24 = 48,048; 1,000 forward lists at capacity 4 × 16 = 64,000; 1,000 backward lists likewise 64,000 |
| chain of 1,000, after 1,024 updates | 176,048 | 1,000 | 176.048 | as above |

The hub's 48.972 bytes per edge is the figure the 1.25× gate of `ultragraph-edge-storage-benchmark`
multiplies: a challenger stays in at most **61.2 bytes per edge** on the hub and **220.1** on the
chain, in every measured configuration.

**Observations.** The hub rows are the targets: a keyed update at 16.5 µs and a removal at
29.7 µs, each the scan of the hub's 100,000-entry list for `(target, key)`, 0.17 ns and 0.30 ns per
entry; the removal's `retain` then shifts what follows the match. A low-degree update is 2.6 ns
and a removal 6 to 13 ns, a list of one entry each. The dynamic walks are unstable: four of the
twelve rows exceed a 15 % spread and give medians, with the shuffled walks at 1,000 nodes
bimodal at 1.8 and 3.4 µs across runs, as the baseline's `inbound_walk` rows were; the match rule
of design D6 widens to the row's spread, so these rows constrain a challenger loosely. The ordered
walks at 1,000 nodes, 0.62 ns per step, come in under their frozen counterparts of change 1
(1.75 ns and 1.03 ns per step) and under the 100-node rows per step; the three runs agree to 3 %,
so it is the fixture, not noise, and it is recorded here without an explanation.

### Group 2: the incumbent behind the storage trait

The `Vec` lists re-measured after group 2 moved them behind the crate-internal `EdgeList` trait
(the build every candidate shares), on 2026-10-16; the logs are `Group2-IncumbentRun-1.txt` to
`Group2-IncumbentRun-3.txt`, the same groups and fixtures as group 1. These rows are the incumbent
every candidate of this change is compared with: the same build, so the trait's cost falls on
both sides.

| Benchmark | n | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---:|---:|---:|---:|---:|---:|
| `keyed_update_low` | 10 | 2.56 ns | 2.61 ns | 2.65 ns | 2.60 ns | 3.3 % |  |
| `keyed_update_low` | 100 | 2.54 ns | 2.63 ns | 2.60 ns | 2.59 ns | 3.8 % |  |
| `keyed_update_low` | 1,000 | 2.69 ns | 2.80 ns | 2.70 ns | 2.73 ns | 4.0 % |  |
| `keyed_update_hub` | 100,000 | 16.23 µs | 16.38 µs | 16.45 µs | 16.36 µs | 1.4 % |  |
| `keyed_remove_low` | 10 | 7.30 ns | 7.46 ns | 7.33 ns | 7.36 ns | 2.1 % |  |
| `keyed_remove_low` | 100 | 7.10 ns | 7.32 ns | 7.48 ns | 7.30 ns | 5.2 % |  |
| `keyed_remove_low` | 1,000 | 11.02 ns | 15.00 ns | 13.89 ns | 13.89 ns (median) | 30.0 % |  |
| `keyed_remove_hub` | 100,000 | 29.29 µs | 30.37 µs | 29.96 µs | 29.87 µs | 3.6 % |  |
| `outbound_walk_dynamic` | 10 | 14.53 ns | 14.90 ns | 14.91 ns | 14.78 ns | 2.6 % | 1.48 ns |
| `outbound_walk_dynamic` | 100 | 298.20 ns | 297.24 ns | 298.87 ns | 298.10 ns | 0.5 % | 2.98 ns |
| `outbound_walk_dynamic` | 1,000 | 613.06 ns | 622.46 ns | 635.01 ns | 623.51 ns | 3.5 % | 0.62 ns |
| `inbound_walk_dynamic` | 10 | 14.54 ns | 14.73 ns | 14.76 ns | 14.67 ns | 1.5 % | 1.47 ns |
| `inbound_walk_dynamic` | 100 | 239.29 ns | 283.26 ns | 238.10 ns | 239.29 ns (median) | 17.8 % | 2.39 ns |
| `inbound_walk_dynamic` | 1,000 | 634.15 ns | 619.45 ns | 633.95 ns | 629.18 ns | 2.3 % | 0.63 ns |
| `outbound_walk_shuffled_dynamic` | 10 | 14.94 ns | 14.76 ns | 14.86 ns | 14.85 ns | 1.2 % | 1.48 ns |
| `outbound_walk_shuffled_dynamic` | 100 | 300.95 ns | 315.27 ns | 299.15 ns | 305.12 ns | 5.3 % | 3.05 ns |
| `outbound_walk_shuffled_dynamic` | 1,000 | 1.69 µs | 1.76 µs | 1.83 µs | 1.76 µs | 8.5 % | 1.76 ns |
| `inbound_walk_shuffled_dynamic` | 10 | 14.42 ns | 14.91 ns | 14.98 ns | 14.77 ns | 3.8 % | 1.48 ns |
| `inbound_walk_shuffled_dynamic` | 100 | 272.31 ns | 278.52 ns | 260.46 ns | 270.43 ns | 6.7 % | 2.70 ns |
| `inbound_walk_shuffled_dynamic` | 1,000 | 2.88 µs | 2.22 µs | 1.82 µs | 2.22 µs (median) | 46.0 % | 2.22 ns |

Memory per edge is unchanged: 48.972 bytes on the hub, 176.048 on the chain of 1,000, built and
after the updates (the `Vec` list behind the trait is the same `Vec`).

**Against group 1.** Sixteen of the twenty rows are within the group-1 row's tolerance (the
larger of 5 % and its spread). `keyed_remove_low` moved up at every size, +11.9 %, +17.0 % and
+8.7 %, from 6.6 to 7.4 ns, 6.2 to 7.3 ns and 12.8 to 13.9 ns: `remove_edge_keyed` now removes
the backward entry by a position scan and a `remove` through the trait where it ran a `retain`,
and the forward removal pays the trait's key extraction on its one-entry list. The hub removal,
the row the decision rule reads, is within 1 %. `outbound_walk_dynamic` at 100 moved +5.1 %,
at the edge of its tolerance, with the 1,000-node rows unchanged; the shuffled walks at 1,000
nodes read 1.8 and 2.2 µs where group 1 read 3.4 µs medians, inside their 50 % spreads, which
is the instability group 1 recorded.

### Group 3: the edge-storage decision

#### The memory gate

Memory per edge of each challenger across its parameter, one run per value of the
`memory_per_edge` group under the challenger's feature (the sweep edits the constant, builds and
runs; deterministic, so one run), on 2026-10-16. The incumbent reads 48.972 bytes per edge on
the hub and 176.048 on the chain of 1,000, built and after the updates, so the 1.25× gate is
**61.2** on the hub and **220.1** on the chain.

| Candidate | Parameter | Hub, built | Hub, updated | Chain of 1,000, built | Chain, updated |
|---|---:|---:|---:|---:|---:|
| snapshot-log arrays | `MIN_LOG` = 4 | 80.973 | 80.973 | 240.112 | 241.568 |
| snapshot-log arrays | 8, 16, 32, 64, 128, 1,024 | 64.973 | 64.973 | 240.112 | 241.392 |
| indexed lists | `HUB_DEGREE` = 8, 16, 32, 64, 128, 256, 1,024, 8,192 | 96.336 | 96.336 | 240.112 | 240.112 |

The parameter moves nothing the gate reads. The hub's 100,000 entries sit in one list, so its
log compacts at the snapshot's length once that exceeds `MIN_LOG`, and its index exists at every
`HUB_DEGREE` below the degree; at `MIN_LOG` = 4 the leaves' four-entry backward logs compact into a
snapshot beside a kept log, 16 bytes more per leaf. The chain's figure is the list header: both
challengers carry 56 bytes per node and direction (two `Vec`s and a count; a `Vec` and an
`Option<BTreeMap>`) against the `Vec` list's 24, which is 64,064 bytes over the chain's 1,000
edges, the whole of its rise from 176.048 to 240.112. On the hub the snapshot-log array adds the
same header cost, 1,600,064 bytes, to a snapshot of the incumbent's size; the index adds its two
map levels, 3,136,368 bytes, on top.

**Neither challenger has a value at or under the gate in every measured configuration, so by the
rule of `ultragraph-edge-storage-benchmark` both are out and the `Vec` lists stay.**

#### The challengers' timings

Measured at the defaults, `MIN_LOG` = 16 and `HUB_DEGREE` = 64, for the record: the gate had
already excluded both, so these rows decide nothing. The snapshot-log arrays ran three times
(`Group3-SnapshotLogRun-1.txt` to `-3.txt`); the runs were hours apart on 2026-10-16 and the
second and third read 1.4 to 1.6 times the first on every row, so every spread exceeds 15 % and
the table gives medians. The indexed lists ran once (`Group3-IndexedRun-1.txt`), in the same
later machine state; the two further runs were stopped once the decision stood.

Snapshot-log arrays, three runs:

| Benchmark | n | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---:|---:|---:|---:|---:|---:|
| `keyed_update_low` | 10 | 10.79 ns | 17.24 ns | 15.23 ns | 15.23 ns (median) | 44.8 % |  |
| `keyed_update_low` | 100 | 10.43 ns | 15.72 ns | 15.19 ns | 15.19 ns (median) | 38.4 % |  |
| `keyed_update_low` | 1,000 | 10.59 ns | 15.68 ns | 15.39 ns | 15.39 ns (median) | 36.6 % |  |
| `keyed_update_hub` | 100,000 | 340.93 ns | 481.67 ns | 493.88 ns | 481.67 ns (median) | 34.9 % |  |
| `keyed_remove_low` | 10 | 10.47 ns | 13.52 ns | 13.94 ns | 13.52 ns (median) | 27.4 % |  |
| `keyed_remove_low` | 100 | 11.32 ns | 15.14 ns | 16.29 ns | 15.14 ns (median) | 34.9 % |  |
| `keyed_remove_low` | 1,000 | 20.95 ns | 33.80 ns | 35.28 ns | 33.80 ns (median) | 47.7 % |  |
| `keyed_remove_hub` | 100,000 | 15.08 µs | 22.49 µs | 12.59 µs | 15.08 µs (median) | 59.2 % |  |
| `outbound_walk_dynamic` | 10 | 438.49 ns | 603.74 ns | 609.54 ns | 603.74 ns (median) | 31.1 % | 60.37 ns |
| `outbound_walk_dynamic` | 100 | 4.30 µs | 6.01 µs | 5.77 µs | 5.77 µs (median) | 31.9 % | 57.75 ns |
| `outbound_walk_dynamic` | 1,000 | 42.67 µs | 59.32 µs | 58.01 µs | 58.01 µs (median) | 31.2 % | 58.01 ns |
| `inbound_walk_dynamic` | 10 | 404.89 ns | 571.41 ns | 577.22 ns | 571.41 ns (median) | 33.3 % | 57.14 ns |
| `inbound_walk_dynamic` | 100 | 4.05 µs | 5.69 µs | 5.54 µs | 5.54 µs (median) | 32.1 % | 55.45 ns |
| `inbound_walk_dynamic` | 1,000 | 40.43 µs | 56.00 µs | 57.56 µs | 56.00 µs (median) | 33.4 % | 56.00 ns |
| `outbound_walk_shuffled_dynamic` | 10 | 437.87 ns | 607.72 ns | 609.94 ns | 607.72 ns (median) | 31.2 % | 60.77 ns |
| `outbound_walk_shuffled_dynamic` | 100 | 4.28 µs | 5.93 µs | 5.84 µs | 5.84 µs (median) | 30.9 % | 58.40 ns |
| `outbound_walk_shuffled_dynamic` | 1,000 | 42.51 µs | 57.68 µs | 58.80 µs | 57.68 µs (median) | 30.7 % | 57.68 ns |
| `inbound_walk_shuffled_dynamic` | 10 | 414.50 ns | 599.30 ns | 585.48 ns | 585.48 ns (median) | 34.7 % | 58.55 ns |
| `inbound_walk_shuffled_dynamic` | 100 | 4.06 µs | 5.85 µs | 5.75 µs | 5.75 µs (median) | 34.3 % | 57.54 ns |
| `inbound_walk_shuffled_dynamic` | 1,000 | 40.63 µs | 56.91 µs | 58.53 µs | 56.91 µs (median) | 34.4 % | 56.91 ns |

Indexed lists, one run, beside the incumbent of group 2:

| `keyed_update_low` | 10 | 4.35 ns | 2.60 ns | 1.67× |
| `keyed_update_low` | 100 | 4.35 ns | 2.59 ns | 1.68× |
| `keyed_update_low` | 1,000 | 4.95 ns | 2.73 ns | 1.81× |
| `keyed_update_hub` | 100,000 | 31.75 ns | 16.36 µs | 0.00× |
| `keyed_remove_low` | 10 | 15.06 ns | 7.36 ns | 2.05× |
| `keyed_remove_low` | 100 | 16.59 ns | 7.30 ns | 2.27× |
| `keyed_remove_low` | 1,000 | 27.96 ns | 13.89 ns | 2.01× |
| `keyed_remove_hub` | 100,000 | 502.79 ns | 29.87 µs | 0.02× |
| `outbound_walk_dynamic` | 10 | 20.42 ns | 14.78 ns | 1.38× |
| `outbound_walk_dynamic` | 100 | 277.80 ns | 298.10 ns | 0.93× |
| `outbound_walk_dynamic` | 1,000 | 4.37 µs | 623.51 ns | 7.01× |
| `inbound_walk_dynamic` | 10 | 20.99 ns | 14.67 ns | 1.43× |
| `inbound_walk_dynamic` | 100 | 291.13 ns | 239.29 ns | 1.22× |
| `inbound_walk_dynamic` | 1,000 | 3.70 µs | 629.18 ns | 5.89× |
| `outbound_walk_shuffled_dynamic` | 10 | 20.68 ns | 14.85 ns | 1.39× |
| `outbound_walk_shuffled_dynamic` | 100 | 441.19 ns | 305.12 ns | 1.45× |
| `outbound_walk_shuffled_dynamic` | 1,000 | 5.95 µs | 1.76 µs | 3.38× |
| `inbound_walk_shuffled_dynamic` | 10 | 20.61 ns | 14.77 ns | 1.40× |
| `inbound_walk_shuffled_dynamic` | 100 | 442.84 ns | 270.43 ns | 1.64× |
| `inbound_walk_shuffled_dynamic` | 1,000 | 6.03 µs | 2.22 µs | 2.71× |

**Against the rule, had the gate passed.** On the hub rows both challengers are clearly faster
than the incumbent's 16.36 µs update and 29.87 µs removal: the snapshot-log arrays at 0.48 µs
and 15.1 µs (medians), the indexed lists at 32 ns and 0.50 µs. Neither matches the incumbent on
traversal: the snapshot-log arrays walk at 56 to 61 ns per step against the incumbent's 0.6 to
3.1 ns, because a read with a non-empty log resolves the log into a list on every call, two sets
and a vector per node visited; the indexed lists read 1.4 to 7 times the incumbent's walk rows
in a run whose machine state also carried the log arrays' later runs 1.5 times above their first,
so that factor is not separable from one run. The snapshot-log arrays also read 4 to 6 times the
incumbent on every low-degree row, the lookup through a log before each write. The traversal rows
alone would have taken the snapshot-log arrays out under the rule; the indexed lists' hub rows
are the one result the gate left on the table.

**Decision (task 3.2).** The `Vec` lists stay. Both challengers, the storage trait they shared and
the features `edge-log` and `edge-index` were removed from `ultragraph` with Marvin's approval
(task 3.3); the dynamic graph is the code of group 1 again, and the five tests that pinned the
log's trigger left with it. The contract tests written for the candidates stay as the keyed-edge
contract of the dynamic state (`tests/types/storage/graph_dynamic/graph_mut_tests.rs`), and the
accounting tests keep the `Vec` list's literals.

### Group 5: the Live decision

Three runs of the `live_*` groups of `benches/benchmarks/bench_live.rs` on 2026-10-16, the logs
`Group5-LiveRun-1.txt` to `Group5-LiveRun-3.txt`, aggregated as above. Fixtures: the keyed
chain of 1,000 and the keyed hub of 100,000 edges. A keyed write is `add_edge` on an edge that
exists, cycling through random edges, so the structure stays; the `dynamic` rows are the
workflow Live replaces, writes in the Dynamic state and `freeze` before each
`strongly_connected_components`, `unfreeze` after.

| Row | Run 1 | Run 2 | Run 3 | Mean | Spread |
|---|---:|---:|---:|---:|---:|
| `live_writes_then_scc/live/chain_1000/1` | 44.53 µs | 44.66 µs | 44.86 µs | 44.69 µs | 0.7 % |
| `live_writes_then_scc/dynamic/chain_1000/1` | 147.33 µs | 144.22 µs | 144.23 µs | 145.26 µs | 2.1 % |
| `live_writes_then_scc/live/chain_1000/16` | 45.00 µs | 44.56 µs | 45.17 µs | 44.91 µs | 1.4 % |
| `live_writes_then_scc/dynamic/chain_1000/16` | 145.67 µs | 143.46 µs | 145.92 µs | 145.02 µs | 1.7 % |
| `live_writes_then_scc/live/chain_1000/256` | 47.54 µs | 47.73 µs | 47.98 µs | 47.75 µs | 0.9 % |
| `live_writes_then_scc/dynamic/chain_1000/256` | 144.92 µs | 145.04 µs | 145.83 µs | 145.26 µs | 0.6 % |
| `live_writes_then_scc/live/hub/1` | 1.040 ms | 1.039 ms | 1.039 ms | 1.039 ms | 0.1 % |
| `live_writes_then_scc/dynamic/hub/1` | 7.670 ms | 7.488 ms | 7.441 ms | 7.533 ms | 3.0 % |
| `live_writes_then_scc/live/hub/16` | 1.051 ms | 1.043 ms | 1.050 ms | 1.048 ms | 0.8 % |
| `live_writes_then_scc/dynamic/hub/16` | 7.987 ms | 7.941 ms | 7.772 ms | 7.900 ms | 2.7 % |
| `live_writes_then_scc/live/hub/256` | 1.087 ms | 1.081 ms | 1.080 ms | 1.083 ms | 0.6 % |
| `live_writes_then_scc/dynamic/hub/256` | 13.112 ms | 13.325 ms | 13.078 ms | 13.172 ms | 1.9 % |
| `live_algorithms/scc/frozen` | 46.15 µs | 45.83 µs | 46.34 µs | 46.10 µs | 1.1 % |
| `live_algorithms/topological_sort/frozen` | 9.41 µs | 9.10 µs | 9.32 µs | 9.28 µs | 3.4 % |
| `live_algorithms/shortest_path_len/frozen` | 3.96 µs | 3.99 µs | 3.98 µs | 3.98 µs | 0.8 % |
| `live_algorithms/scc/live` | 44.12 µs | 44.03 µs | 44.29 µs | 44.15 µs | 0.6 % |
| `live_algorithms/topological_sort/live` | 7.93 µs | 7.82 µs | 8.02 µs | 7.92 µs | 2.5 % |
| `live_algorithms/shortest_path_len/live` | 3.15 µs | 3.17 µs | 3.15 µs | 3.16 µs | 0.9 % |
| `live_update/in_place` | 24.41 ns | 26.20 ns | 23.69 ns | 24.77 ns | 10.1 % |
| `live_update/rebuild` | 802.30 ns | 787.00 ns | 791.77 ns | 793.69 ns | 1.9 % |
| `live_transitions/make_live/from_dynamic` | 68.14 µs | 67.07 µs | 68.52 µs | 67.91 µs | 2.1 % |
| `live_transitions/make_live/from_frozen` | 1.24 µs | 1.06 µs | 1.26 µs | 1.24 µs (median) | 16.7 % |
| `live_transitions/freeze/from_live` | 6.81 µs | 7.09 µs | 7.08 µs | 6.99 µs | 4.1 % |
| `live_transitions/unfreeze/from_live` | 29.90 µs | 30.71 µs | 29.98 µs | 30.19 µs | 2.7 % |
| `live_transitions/freeze_clone/dynamic` | 96.39 µs | 97.63 µs | 100.63 µs | 98.22 µs | 4.3 % |
| `live_transitions/freeze_clone/frozen` | 2.92 µs | 2.36 µs | 2.82 µs | 2.82 µs (median) | 20.7 % |
| `live_transitions/freeze_clone/live` | 9.70 µs | 9.68 µs | 9.60 µs | 9.66 µs | 1.1 % |

| Fixture | k | Live mean | Dynamic path mean | Live / Dynamic | Every run faster | Margin beyond spread | Clearly faster |
|---|---:|---:|---:|---:|---|---|---|
| chain_1000 | 1 | 44.69 µs | 145.26 µs | 0.31 | yes | yes | **yes** |
| chain_1000 | 16 | 44.91 µs | 145.02 µs | 0.31 | yes | yes | **yes** |
| chain_1000 | 256 | 47.75 µs | 145.26 µs | 0.33 | yes | yes | **yes** |
| hub | 1 | 1.039 ms | 7.533 ms | 0.14 | yes | yes | **yes** |
| hub | 16 | 1.048 ms | 7.900 ms | 0.13 | yes | yes | **yes** |
| hub | 256 | 1.083 ms | 13.172 ms | 0.08 | yes | yes | **yes** |

**Decision (task 5.2).** Every one of the six rows the rule reads holds, each of Live's three
runs below each of the other path's and the mean below by far more than the spread: **the Live
state stays**, and group 6 builds the maintained views on its layout.

**What the rows say.** On the chain the freeze-per-analysis path is the algorithm (46 µs
frozen) plus a freeze and an unfreeze of 1,000 nodes (about 100 µs, 68 µs of it the sort and
placement `make_live/from_dynamic` also pays); Live pays the algorithm alone, 44 to 48 µs, the
256 writes adding 3 µs. On the hub the freeze of 100,000 edges dominates (7.5 ms against 1.04 ms
for the algorithm), and at k = 256 the Dynamic writes themselves add 5.6 ms, the scan of the
hub's list per write, where Live's binary searches add 44 µs. The three algorithms read a
little faster on the Live rows than on the frozen CSR (SCC 44.2 against 46.1 µs, the sort 7.9
against 9.3, the path 3.2 against 4.0); the Live read does more work per edge, a mask test per
entry, so the difference has no verified explanation and is recorded as measured.

**What the rows do not say.** The `dynamic` rows carry a `freeze` and an `unfreeze` per
analysis because the workflow they measure interleaves writes with analyses, and a frozen graph
takes no write; a workflow that writes once and analyses many times pays the freeze once, and for
it the `live_algorithms` rows are the comparison: Live and Frozen within a few percent. The
`live` rows exclude `make_live`, paid once; its cost is in `live_transitions`. An in-place insert into a row with a spare slot is
25 ns; an insert that rebuilds a row of capacity one at the arena's end is 0.79 µs on a 1,000-node
chain, the clone of the arenas per batch excluded. `make_live` from Frozen is 1.2 µs for 1,000
positions (the row records), `freeze` from Live 7 µs (a pack with no sort), `unfreeze` from Live
30 µs; `freeze_clone` costs the state's freeze plus the copy, 98 µs from Dynamic, 9.7 µs from Live,
2.8 µs from Frozen.

### Group 7: the final measurement

Three runs of the whole `cargo bench -p ultragraph`, every group, on 2026-10-16 between 17:10
and 18:20, the logs `Group7-FinalRun-1.txt` to `Group7-FinalRun-3.txt`. Each row is compared
with the incumbent it has: the after-benchmark of change 1 for the rows that existed then
(`../BenchAfterChange1StableId/BenchAfterChange1StableId.md`, with the review re-measurement of
its `add_edge`, `remove_node` and `remove_edge` rows), the group-2 incumbent for the storage rows,
and no incumbent for the Live and view rows, which are new. The verdict is design D6's: within
spread when the mean sits inside the incumbent's mean times one plus the larger of 5 % and the
incumbent row's spread, faster or slower otherwise. The `bridges/10` incumbent is the median of
change 1's after-run, 1.21 µs, where the table below shows its per-element figure.

| Row | Run 1 | Run 2 | Run 3 | Mean | Spread | Incumbent | Its spread | Δ | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `build_graph/10` | 327.97 ns | 328.19 ns | 311.98 ns | 322.71 ns | 5.0 % | 200.93 ns | 6.1 % | +60.6 % | slower |
| `build_graph/100` | 3.05 µs | 2.98 µs | 3.01 µs | 3.01 µs | 2.3 % | 1.69 µs | 7.0 % | +78.2 % | slower |
| `build_graph/1000` | 30.13 µs | 29.64 µs | 28.54 µs | 29.44 µs | 5.4 % | 16.18 µs | 4.8 % | +81.9 % | slower |
| `add_node/10` | 6.91 ns | 7.05 ns | 6.52 ns | 6.83 ns | 7.8 % | 5.09 ns | 1.5 % | +34.2 % | slower |
| `add_node/100` | 7.80 ns | 8.09 ns | 7.78 ns | 7.89 ns | 4.0 % | 5.99 ns | 1.4 % | +31.7 % | slower |
| `add_node/1000` | 12.78 ns | 12.87 ns | 11.83 ns | 12.49 ns | 8.4 % | 8.66 ns | 2.0 % | +44.3 % | slower |
| `get_node/10` | 2.39 ns | 2.41 ns | 2.28 ns | 2.36 ns | 5.7 % | 2.38 ns | 2.5 % | -0.8 % | within spread |
| `get_node/100` | 2.41 ns | 2.40 ns | 2.37 ns | 2.39 ns | 1.4 % | 2.32 ns | 0.4 % | +3.2 % | within spread |
| `get_node/1000` | 2.40 ns | 2.40 ns | 2.27 ns | 2.36 ns | 5.5 % | 2.34 ns | 3.0 % | +0.7 % | within spread |
| `articulation_points/10` | 1.67 µs | 1.60 µs | 1.58 µs | 1.62 µs | 5.9 % | 1.32 µs | 12.2 % | +22.4 % | slower |
| `articulation_points/100` | 11.95 µs | 12.15 µs | 11.29 µs | 11.80 µs | 7.3 % | 10.55 µs | 1.1 % | +11.8 % | slower |
| `articulation_points/1000` | 142.73 µs | 143.53 µs | 138.12 µs | 141.46 µs | 3.8 % | 134.45 µs | 8.2 % | +5.2 % | within spread |
| `articulation_points/5000` | 955.69 µs | 959.33 µs | 919.48 µs | 944.83 µs | 4.2 % | 902.21 µs | 12.8 % | +4.7 % | within spread |
| `bridges/10` | 1.48 µs | 1.56 µs | 1.35 µs | 1.46 µs | 14.1 % | 40.32 ns | 15.1 % | +3524.8 % | slower |
| `bridges/100` | 11.70 µs | 11.69 µs | 10.97 µs | 11.45 µs | 6.4 % | 10.44 µs | 0.7 % | +9.7 % | slower |
| `bridges/1000` | 141.69 µs | 143.14 µs | 136.05 µs | 140.29 µs | 5.1 % | 127.97 µs | 3.3 % | +9.6 % | slower |
| `bridges/5000` | 927.38 µs | 931.36 µs | 895.02 µs | 917.92 µs | 4.0 % | 863.27 µs | 3.6 % | +6.3 % | slower |
| `biconnected_components/10` | 2.48 µs | 2.59 µs | 2.39 µs | 2.48 µs | 7.8 % | 1.96 µs | 4.6 % | +26.8 % | slower |
| `biconnected_components/100` | 19.55 µs | 19.47 µs | 18.51 µs | 19.18 µs | 5.4 % | 17.27 µs | 2.9 % | +11.1 % | slower |
| `biconnected_components/1000` | 216.36 µs | 219.37 µs | 200.85 µs | 212.19 µs | 8.7 % | 188.29 µs | 2.8 % | +12.7 % | slower |
| `biconnected_components/5000` | 1.330 ms | 1.310 ms | 1.251 ms | 1.297 ms | 6.1 % | 1.177 ms | 3.8 % | +10.2 % | slower |
| `add_edge/10` | 19.17 ns | 18.92 ns | 17.95 ns | 18.68 ns | 6.5 % | 18.26 ns | 2.4 % | +2.3 % | within spread |
| `add_edge/100` | 50.33 ns | 51.58 ns | 48.13 ns | 50.01 ns | 6.9 % | 52.09 ns | 3.6 % | -4.0 % | within spread |
| `add_edge/1000` | 344.79 ns | 338.16 ns | 333.29 ns | 338.75 ns | 3.4 % | 352.80 ns | 0.5 % | -4.0 % | within spread |
| `contains_edge_dynamic/10` | 2.97 ns | 3.01 ns | 2.85 ns | 2.94 ns | 5.6 % | 2.85 ns | 2.0 % | +3.2 % | within spread |
| `contains_edge_dynamic/100` | 21.18 ns | 21.52 ns | 20.31 ns | 21.01 ns | 5.8 % | 21.15 ns | 1.2 % | -0.7 % | within spread |
| `contains_edge_dynamic/1000` | 151.02 ns | 153.04 ns | 144.70 ns | 149.59 ns | 5.6 % | 146.42 ns | 1.7 % | +2.2 % | within spread |
| `contains_edge_frozen/10` | 5.11 ns | 5.19 ns | 4.94 ns | 5.08 ns | 4.9 % | 5.06 ns | 2.0 % | +0.4 % | within spread |
| `contains_edge_frozen/100` | 5.15 ns | 5.34 ns | 5.03 ns | 5.17 ns | 6.0 % | 4.90 ns | 1.8 % | +5.5 % | slower |
| `contains_edge_frozen/1000` | 7.34 ns | 7.47 ns | 7.07 ns | 7.29 ns | 5.5 % | 6.99 ns | 1.7 % | +4.3 % | within spread |
| `get_edges/10` | 20.41 ns | 21.53 ns | 20.25 ns | 20.73 ns | 6.2 % | 11.70 ns | 3.5 % | +77.2 % | slower |
| `get_edges/100` | 40.48 ns | 41.14 ns | 39.05 ns | 40.22 ns | 5.2 % | 38.88 ns | 1.0 % | +3.5 % | within spread |
| `get_edges/1000` | 280.17 ns | 280.57 ns | 271.05 ns | 277.26 ns | 3.4 % | 278.25 ns | 5.7 % | -0.4 % | within spread |
| `outbound_walk/10` | 13.13 ns | 13.42 ns | 12.86 ns | 13.13 ns | 4.3 % | 12.66 ns | 4.7 % | +3.7 % | within spread |
| `outbound_walk/100` | 205.66 ns | 210.76 ns | 199.89 ns | 205.44 ns | 5.3 % | 214.26 ns | 4.5 % | -4.1 % | within spread |
| `outbound_walk/1000` | 1.84 µs | 1.85 µs | 1.78 µs | 1.83 µs | 3.6 % | 1.75 µs | 1.1 % | +4.3 % | within spread |
| `inbound_walk/10` | 13.46 ns | 13.43 ns | 12.75 ns | 13.22 ns | 5.4 % | 12.55 ns | 2.1 % | +5.3 % | slower |
| `inbound_walk/100` | 131.74 ns | 210.14 ns | 204.62 ns | 204.62 ns (median) | 43.0 % | 202.03 ns | 0.8 % | +1.3 % | within spread |
| `inbound_walk/1000` | 997.28 ns | 935.59 ns | 842.01 ns | 935.59 ns (median) | 16.8 % | 1.03 µs | 6.4 % | -9.2 % | faster |
| `outbound_walk_shuffled/10` | 13.51 ns | 13.46 ns | 13.10 ns | 13.35 ns | 3.0 % | 12.23 ns | 0.2 % | +9.2 % | slower |
| `outbound_walk_shuffled/100` | 213.68 ns | 211.44 ns | 199.57 ns | 208.23 ns | 6.8 % | 212.60 ns | 2.2 % | -2.1 % | within spread |
| `outbound_walk_shuffled/1000` | 2.26 µs | 2.26 µs | 2.15 µs | 2.22 µs | 5.3 % | 2.14 µs | 0.9 % | +3.9 % | within spread |
| `inbound_walk_shuffled/10` | 13.87 ns | 13.66 ns | 12.82 ns | 13.45 ns | 7.8 % | 12.47 ns | 3.1 % | +7.9 % | slower |
| `inbound_walk_shuffled/100` | 217.93 ns | 211.50 ns | 199.93 ns | 209.79 ns | 8.6 % | 200.89 ns | 1.5 % | +4.4 % | within spread |
| `inbound_walk_shuffled/1000` | 2.22 µs | 2.27 µs | 2.13 µs | 2.20 µs | 6.2 % | 2.16 µs | 1.6 % | +2.1 % | within spread |
| `update_node/10` | 2.46 ns | 2.45 ns | 2.30 ns | 2.40 ns | 6.4 % | 2.27 ns | 1.9 % | +5.8 % | slower |
| `update_node/100` | 2.45 ns | 2.46 ns | 2.31 ns | 2.40 ns | 6.2 % | 2.25 ns | 0.2 % | +6.9 % | slower |
| `update_node/1000` | 2.44 ns | 2.39 ns | 2.35 ns | 2.39 ns | 3.8 % | 2.25 ns | 1.1 % | +6.4 % | slower |
| `remove_node/10` | 23.85 ns | 23.25 ns | 22.63 ns | 23.25 ns | 5.2 % | 23.58 ns | 4.5 % | -1.4 % | within spread |
| `remove_node/100` | 25.70 ns | 26.97 ns | 24.57 ns | 25.75 ns | 9.3 % | 24.99 ns | 1.5 % | +3.0 % | within spread |
| `remove_node/1000` | 31.46 ns | 32.21 ns | 29.58 ns | 31.08 ns | 8.5 % | 30.19 ns | 8.8 % | +3.0 % | within spread |
| `remove_edge/10` | 19.04 ns | 19.06 ns | 19.02 ns | 19.04 ns | 0.2 % | 18.18 ns | 4.3 % | +4.7 % | within spread |
| `remove_edge/100` | 50.59 ns | 52.16 ns | 47.92 ns | 50.23 ns | 8.4 % | 47.29 ns | 2.4 % | +6.2 % | slower |
| `remove_edge/1000` | 327.43 ns | 325.45 ns | 308.72 ns | 320.53 ns | 5.8 % | 328.34 ns | 2.9 % | -2.4 % | within spread |
| `freeze/10` | 683.85 ns | 669.33 ns | 651.56 ns | 668.25 ns | 4.8 % | 496.64 ns | 0.5 % | +34.6 % | slower |
| `freeze/100` | 5.41 µs | 5.30 µs | 5.11 µs | 5.28 µs | 5.7 % | 4.05 µs | 2.0 % | +30.3 % | slower |
| `freeze/1000` | 53.27 µs | 50.36 µs | 50.92 µs | 51.52 µs | 5.6 % | 39.76 µs | 3.7 % | +29.6 % | slower |
| `unfreeze/10` | 303.62 ns | 282.43 ns | 283.84 ns | 289.96 ns | 7.3 % | 203.98 ns | 3.5 % | +42.2 % | slower |
| `unfreeze/100` | 2.39 µs | 2.26 µs | 2.26 µs | 2.30 µs | 5.6 % | 1.49 µs | 0.2 % | +54.5 % | slower |
| `unfreeze/1000` | 21.48 µs | 20.68 µs | 19.84 µs | 20.67 µs | 7.9 % | 14.07 µs | 4.6 % | +46.9 % | slower |
| `keyed_update_low/10` | 3.98 ns | 3.77 ns | 3.76 ns | 3.84 ns | 5.7 % | 2.60 ns | 3.3 % | +47.5 % | slower |
| `keyed_update_low/100` | 4.00 ns | 3.76 ns | 3.77 ns | 3.85 ns | 6.2 % | 2.59 ns | 3.8 % | +48.5 % | slower |
| `keyed_update_low/1000` | 4.10 ns | 4.01 ns | 3.88 ns | 4.00 ns | 5.6 % | 2.73 ns | 4.0 % | +46.4 % | slower |
| `keyed_update_hub/100000` | 16.57 µs | 15.67 µs | 15.67 µs | 15.97 µs | 5.6 % | 16.36 µs | 1.4 % | -2.4 % | within spread |
| `keyed_remove_low/10` | 7.06 ns | 6.67 ns | 6.68 ns | 6.80 ns | 5.8 % | 7.36 ns | 2.1 % | -7.6 % | faster |
| `keyed_remove_low/100` | 7.12 ns | 6.93 ns | 6.70 ns | 6.92 ns | 6.0 % | 7.30 ns | 5.2 % | -5.3 % | faster |
| `keyed_remove_low/1000` | 13.97 ns | 12.29 ns | 12.09 ns | 12.78 ns | 14.7 % | 13.89 ns | 30.0 % | -8.0 % | within spread |
| `keyed_remove_hub/100000` | 30.04 µs | 28.52 µs | 28.30 µs | 28.96 µs | 6.0 % | 29.87 µs | 3.6 % | -3.1 % | within spread |
| `outbound_walk_dynamic/10` | 14.74 ns | 14.13 ns | 14.05 ns | 14.31 ns | 4.9 % | 14.78 ns | 2.6 % | -3.2 % | within spread |
| `outbound_walk_dynamic/100` | 285.85 ns | 270.33 ns | 276.04 ns | 277.41 ns | 5.6 % | 298.10 ns | 0.5 % | -6.9 % | faster |
| `outbound_walk_dynamic/1000` | 1.13 µs | 1.00 µs | 957.32 ns | 1.00 µs (median) | 17.1 % | 623.51 ns | 3.5 % | +60.7 % | slower |
| `inbound_walk_dynamic/10` | 14.60 ns | 13.98 ns | 14.19 ns | 14.26 ns | 4.4 % | 14.67 ns | 1.5 % | -2.8 % | within spread |
| `inbound_walk_dynamic/100` | 348.12 ns | 332.45 ns | 299.98 ns | 326.85 ns | 14.7 % | 239.29 ns | 17.8 % | +36.6 % | slower |
| `inbound_walk_dynamic/1000` | 3.04 µs | 2.43 µs | 3.24 µs | 3.04 µs (median) | 27.9 % | 629.18 ns | 2.3 % | +383.1 % | slower |
| `outbound_walk_shuffled_dynamic/10` | 14.64 ns | 14.06 ns | 13.99 ns | 14.23 ns | 4.6 % | 14.85 ns | 1.2 % | -4.2 % | within spread |
| `outbound_walk_shuffled_dynamic/100` | 324.99 ns | 302.20 ns | 300.39 ns | 309.19 ns | 8.0 % | 305.12 ns | 5.3 % | +1.3 % | within spread |
| `outbound_walk_shuffled_dynamic/1000` | 3.43 µs | 3.22 µs | 3.22 µs | 3.29 µs | 6.4 % | 1.76 µs | 8.5 % | +86.8 % | slower |
| `inbound_walk_shuffled_dynamic/10` | 14.92 ns | 13.93 ns | 13.97 ns | 14.27 ns | 6.9 % | 14.77 ns | 3.8 % | -3.4 % | within spread |
| `inbound_walk_shuffled_dynamic/100` | 354.68 ns | 331.07 ns | 302.55 ns | 331.07 ns (median) | 15.8 % | 270.43 ns | 6.7 % | +22.4 % | slower |
| `inbound_walk_shuffled_dynamic/1000` | 3.40 µs | 3.21 µs | 3.63 µs | 3.41 µs | 12.5 % | 2.22 µs | 46.0 % | +53.7 % | slower |
| `memory_per_edge/hub_built` | 10.58 µs | 10.00 µs | 10.04 µs | 10.21 µs | 5.6 % | new | | | new |
| `memory_per_edge/hub_updated` | 10.65 µs | 10.05 µs | 10.21 µs | 10.30 µs | 5.9 % | new | | | new |
| `memory_per_edge/chain_1000_built` | 223.08 ns | 210.94 ns | 213.03 ns | 215.68 ns | 5.6 % | new | | | new |
| `memory_per_edge/chain_1000_updated` | 222.00 ns | 210.14 ns | 209.39 ns | 213.84 ns | 5.9 % | new | | | new |
| `live_writes_then_scc/live/chain_1000/1` | 33.01 µs | 30.91 µs | 31.06 µs | 31.66 µs | 6.6 % | new | | | new |
| `live_writes_then_scc/dynamic/chain_1000/1` | 105.74 µs | 99.18 µs | 99.43 µs | 101.45 µs | 6.5 % | new | | | new |
| `live_writes_then_scc/live/chain_1000/16` | 32.99 µs | 31.14 µs | 31.30 µs | 31.81 µs | 5.8 % | new | | | new |
| `live_writes_then_scc/dynamic/chain_1000/16` | 107.10 µs | 100.60 µs | 99.52 µs | 102.41 µs | 7.4 % | new | | | new |
| `live_writes_then_scc/live/chain_1000/256` | 35.80 µs | 33.58 µs | 33.77 µs | 34.38 µs | 6.5 % | new | | | new |
| `live_writes_then_scc/dynamic/chain_1000/256` | 109.97 µs | 101.06 µs | 101.76 µs | 104.26 µs | 8.5 % | new | | | new |
| `live_writes_then_scc/live/hub/1` | 774.00 µs | 734.91 µs | 730.19 µs | 746.37 µs | 5.9 % | new | | | new |
| `live_writes_then_scc/dynamic/hub/1` | 5.391 ms | 5.441 ms | 5.722 ms | 5.518 ms | 6.0 % | new | | | new |
| `live_writes_then_scc/live/hub/16` | 782.81 µs | 749.16 µs | 734.79 µs | 755.59 µs | 6.4 % | new | | | new |
| `live_writes_then_scc/dynamic/hub/16` | 5.633 ms | 5.411 ms | 5.772 ms | 5.605 ms | 6.4 % | new | | | new |
| `live_writes_then_scc/live/hub/256` | 812.30 µs | 756.81 µs | 759.85 µs | 776.32 µs | 7.1 % | new | | | new |
| `live_writes_then_scc/dynamic/hub/256` | 9.683 ms | 9.134 ms | 9.217 ms | 9.345 ms | 5.9 % | new | | | new |
| `live_algorithms/scc/frozen` | 33.88 µs | 31.59 µs | 31.69 µs | 32.39 µs | 7.0 % | new | | | new |
| `live_algorithms/topological_sort/frozen` | 7.32 µs | 6.85 µs | 6.90 µs | 7.02 µs | 6.8 % | new | | | new |
| `live_algorithms/shortest_path_len/frozen` | 4.18 µs | 4.04 µs | 3.97 µs | 4.06 µs | 5.1 % | new | | | new |
| `live_algorithms/scc/live` | 32.03 µs | 30.93 µs | 31.10 µs | 31.35 µs | 3.5 % | new | | | new |
| `live_algorithms/topological_sort/live` | 5.39 µs | 5.11 µs | 5.10 µs | 5.20 µs | 5.6 % | new | | | new |
| `live_algorithms/shortest_path_len/live` | 2.93 µs | 2.73 µs | 2.84 µs | 2.84 µs | 7.1 % | new | | | new |
| `live_update/in_place` | 18.73 ns | 17.67 ns | 17.50 ns | 17.97 ns | 6.8 % | new | | | new |
| `live_update/rebuild` | 598.73 ns | 543.46 ns | 555.42 ns | 565.87 ns | 9.8 % | new | | | new |
| `live_transitions/make_live/from_dynamic` | 48.78 µs | 46.91 µs | 47.59 µs | 47.76 µs | 3.9 % | new | | | new |
| `live_transitions/make_live/from_frozen` | 780.96 ns | 748.59 ns | 749.62 ns | 759.72 ns | 4.3 % | new | | | new |
| `live_transitions/freeze/from_live` | 5.03 µs | 4.85 µs | 4.83 µs | 4.90 µs | 4.1 % | new | | | new |
| `live_transitions/unfreeze/from_live` | 21.62 µs | 21.04 µs | 20.82 µs | 21.16 µs | 3.8 % | new | | | new |
| `live_transitions/freeze_clone/dynamic` | 70.80 µs | 67.03 µs | 67.56 µs | 68.47 µs | 5.5 % | new | | | new |
| `live_transitions/freeze_clone/frozen` | 1.70 µs | 1.78 µs | 1.59 µs | 1.69 µs | 11.1 % | new | | | new |
| `live_transitions/freeze_clone/live` | 6.64 µs | 6.40 µs | 6.24 µs | 6.43 µs | 6.4 % | new | | | new |
| `view_upkeep/keyed_write/dynamic/0` | 4.45 ns | 4.14 ns | 4.11 ns | 4.24 ns | 8.1 % | new | | | new |
| `view_upkeep/keyed_write/dynamic/1` | 10.69 ns | 9.85 ns | 9.85 ns | 10.13 ns | 8.2 % | new | | | new |
| `view_upkeep/keyed_write/dynamic/4` | 40.71 ns | 38.74 ns | 37.91 ns | 39.12 ns | 7.1 % | new | | | new |
| `view_upkeep/keyed_write/dynamic/16` | 159.84 ns | 151.36 ns | 148.71 ns | 153.30 ns | 7.3 % | new | | | new |
| `view_upkeep/keyed_write/live/0` | 8.01 ns | 7.79 ns | 7.76 ns | 7.85 ns | 3.2 % | new | | | new |
| `view_upkeep/keyed_write/live/1` | 14.61 ns | 14.30 ns | 13.82 ns | 14.24 ns | 5.5 % | new | | | new |
| `view_upkeep/keyed_write/live/4` | 45.96 ns | 43.96 ns | 43.03 ns | 44.32 ns | 6.6 % | new | | | new |
| `view_upkeep/keyed_write/live/16` | 169.32 ns | 158.65 ns | 158.73 ns | 162.23 ns | 6.6 % | new | | | new |
| `view_upkeep/update_node/dynamic/0` | 2.34 ns | 2.29 ns | 2.32 ns | 2.32 ns | 2.3 % | new | | | new |
| `view_upkeep/update_node/dynamic/1` | 4.75 ns | 4.55 ns | 4.52 ns | 4.61 ns | 5.0 % | new | | | new |
| `view_upkeep/update_node/dynamic/4` | 6.16 ns | 5.87 ns | 5.88 ns | 5.97 ns | 5.0 % | new | | | new |
| `view_upkeep/update_node/dynamic/16` | 16.45 ns | 15.77 ns | 15.74 ns | 15.98 ns | 4.5 % | new | | | new |
| `view_upkeep/update_node/live/0` | 2.45 ns | 2.34 ns | 2.38 ns | 2.39 ns | 4.6 % | new | | | new |
| `view_upkeep/update_node/live/1` | 4.77 ns | 4.56 ns | 4.57 ns | 4.64 ns | 4.6 % | new | | | new |
| `view_upkeep/update_node/live/4` | 7.05 ns | 6.65 ns | 6.76 ns | 6.82 ns | 5.9 % | new | | | new |
| `view_upkeep/update_node/live/16` | 17.77 ns | 16.75 ns | 17.03 ns | 17.18 ns | 5.9 % | new | | | new |


#### The view-upkeep rows

The cost a mutation pays for its maintained views, on the keyed chain of 1,000 nodes, every
view passing every node and edge:

| Mutation | State | 0 views | 1 view | 4 views | 16 views | Per view |
|---|---|---:|---:|---:|---:|---:|
| keyed write (a replacement) | Dynamic | 4.24 ns | 10.13 ns | 39.12 ns | 153.30 ns | about 9.3 ns |
| keyed write (a replacement) | Live | 7.85 ns | 14.24 ns | 44.32 ns | 162.23 ns | about 9.6 ns |
| payload update, node kept | Dynamic | 2.32 ns | 4.61 ns | 5.97 ns | 15.98 ns | about 0.9 ns |
| payload update, node kept | Live | 2.39 ns | 4.64 ns | 6.82 ns | 17.18 ns | about 0.9 ns |

A replacement costs each view a binary search in two rows and two payload writes; a payload
update that keeps the node in scope costs each view one node test and a comparison of the
result. The first view costs more than the next ones, the clone of the payload the hooks make
once there is any view.

#### What the incumbent comparison cannot say, and the A/B that can

Against the incumbents, 36 rows are within spread, 40 slower, 4 faster and 47 new. The
comparison mixes the code with the machine: the Live rows read 30 % below group 5's with no
change to the code they measure, and `freeze` and `unfreeze` read 30 to 55 % above change 1's
after-run with no change to theirs since group 2. So the commit before the Live state
(c397d4d31) was built from a `git archive` in a scratch directory and run directly before the
current tree on the shared groups, `Group7-AB-Old.txt` and `Group7-AB-New.txt`, the pair read by
`ab.py`. Of 84 shared rows, 30 moved beyond 5 %:

| Rows | Move | What the diff holds |
|---|---:|---|
| `inbound_walk_dynamic/100`, `/1000`; `outbound_walk_dynamic/1000` | +258 %, +396 %; +60 % | the container's traversal iterator is an enum over the states, and its Live variant, a zip-filter-map chain over a row, sets the enum's size for the Dynamic walk too; the walk builds one per step |
| `add_node` /10 to /1000; `build_graph` | +16 to +42 %; +21 to +28 % | the view hook read the node back and iterated an empty view list on every graph |
| `keyed_update_low` | +43 to +54 % | `add_edge` tested for views and built an `Option` around the payload clone on every call |
| `articulation_points/10`, `bridges` /10 to /1000, `biconnected_components`, `keyed_remove_low` | +6 to +22 % | nothing: the frozen algorithms and the dynamic removal are unchanged between the trees |
| `inbound_walk/1000` | -33 % | nothing: the frozen CSR walk is unchanged |

Two fixes followed, in the tree but not in these logs: `RowEntries`, one row read through a mask
as two slices and the mask closure, which `LiveGraph` and the views return in place of the chain;
and the hooks returning before any read or clone when the graph maintains no view. The suite,
clippy and `cargo mutants` over the two files (19 mutants, 18 caught, 1 unviable) pass. The
re-measurement was stopped before it finished; `../BenchCommands.md` holds the commands to run
it, and the rows above are the figures of record until it is.
