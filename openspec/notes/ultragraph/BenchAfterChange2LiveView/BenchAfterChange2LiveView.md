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
