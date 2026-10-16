/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The workloads the Live decision of `ultragraph-live-state-maintained-views` rests on, and
//! the rows that explain it: the Live state against the workflow it replaces, writes in the
//! Dynamic state followed by a freeze before each analysis.
//!
//! Fixtures: the keyed chain of `LARGE` nodes and the keyed hub of `bench_storage` (25,000
//! leaves, four kinds each, 100,000 edges out of one node), each built dynamic and made live
//! where the row needs it. A keyed write is `add_edge` on an edge that exists, so the structure
//! stays and a benchmark can iterate without rebuilding its fixture; the writes cycle through
//! precomputed random edges.
//!
//! Groups:
//! - `live_writes_then_scc`: k keyed writes, then `strongly_connected_components`, on the Live
//!   graph (`live/<fixture>/<k>`) and as Dynamic → `freeze` → the algorithm → `unfreeze`
//!   (`dynamic/<fixture>/<k>`), the rows the rule of `ultragraph-edge-storage-benchmark` reads.
//! - `live_algorithms`: one algorithm on an unchanged graph, Frozen against Live: the price of
//!   reading rows through the mask.
//! - `live_update`: a keyed insert into a row with spare capacity (in place) and into a full row
//!   (a rebuild at the arena's end), on a fresh clone per batch.
//! - `live_transitions`: `make_live` from Dynamic and from Frozen, `freeze` and `unfreeze` from
//!   Live, and `freeze_clone` from each state, on a fresh clone per batch.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group};
use std::hint::black_box;
use ultragraph::*;

use crate::benchmarks::data::{Data, Kind};
use crate::benchmarks::fields::{INDEX_COUNT, LARGE};
use crate::benchmarks::utils::{
    KINDS, build_keyed_chain, build_keyed_hub, kind_of, random_indices,
};

const HUB: usize = 0;
const HUB_LEAVES: usize = 25_000;
/// Samples for the hub rows, each iteration a freeze or a walk over 100,000 edges.
const HUB_SAMPLES: usize = 20;
/// The write counts the rule names.
const WRITE_COUNTS: [usize; 3] = [1, 16, 256];

/// A keyed write of the chain: the payload of edge `i → i + 1` written again.
fn chain_write(g: &mut UltraGraphWeighted<Data, Kind>, i: usize) {
    g.add_edge(black_box(i), i + 1, kind_of(i))
        .expect("Failed to write edge");
}

/// A keyed write of the hub: the payload of edge `hub → leaf` of `kind` written again.
fn hub_write(g: &mut UltraGraphWeighted<Data, Kind>, leaf: usize, kind: u8) {
    g.add_edge(black_box(HUB), leaf, Kind(kind))
        .expect("Failed to write edge");
}

// --- k writes, then one analysis -------------------------------------------------------------

/// `k` writes then `strongly_connected_components` on the Live graph, and the same writes
/// followed by `freeze`, the algorithm and `unfreeze` on the Dynamic graph.
fn bench_writes_then_scc(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_writes_then_scc");

    // The chain of 1,000.
    let sources = random_indices(0..LARGE);
    for k in WRITE_COUNTS {
        let mut live = build_keyed_chain(LARGE);
        live.make_live();
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::new("live/chain_1000", k), &k, |b, &k| {
            b.iter(|| {
                for _ in 0..k {
                    cursor = (cursor + 1) & (INDEX_COUNT - 1);
                    chain_write(&mut live, sources[cursor]);
                }
                black_box(live.strongly_connected_components().expect("Failed SCC"))
            })
        });
        let mut dynamic = build_keyed_chain(LARGE);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::new("dynamic/chain_1000", k), &k, |b, &k| {
            b.iter(|| {
                for _ in 0..k {
                    cursor = (cursor + 1) & (INDEX_COUNT - 1);
                    chain_write(&mut dynamic, sources[cursor]);
                }
                dynamic.freeze();
                let components = dynamic.strongly_connected_components().expect("Failed SCC");
                dynamic.unfreeze();
                black_box(components)
            })
        });
    }

    // The hub.
    group.sample_size(HUB_SAMPLES);
    let leaves = random_indices(1..HUB_LEAVES + 1);
    let kinds = random_indices(0..KINDS as usize);
    for k in WRITE_COUNTS {
        let mut live = build_keyed_hub(HUB_LEAVES);
        live.make_live();
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::new("live/hub", k), &k, |b, &k| {
            b.iter(|| {
                for _ in 0..k {
                    cursor = (cursor + 1) & (INDEX_COUNT - 1);
                    hub_write(&mut live, leaves[cursor], kinds[cursor] as u8);
                }
                black_box(live.strongly_connected_components().expect("Failed SCC"))
            })
        });
        let mut dynamic = build_keyed_hub(HUB_LEAVES);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::new("dynamic/hub", k), &k, |b, &k| {
            b.iter(|| {
                for _ in 0..k {
                    cursor = (cursor + 1) & (INDEX_COUNT - 1);
                    hub_write(&mut dynamic, leaves[cursor], kinds[cursor] as u8);
                }
                dynamic.freeze();
                let components = dynamic.strongly_connected_components().expect("Failed SCC");
                dynamic.unfreeze();
                black_box(components)
            })
        });
    }
    group.finish();
}

// --- One algorithm, Frozen against Live --------------------------------------------------------

/// Three algorithms on the unchanged chain of 1,000, frozen and live.
fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_algorithms");
    let mut frozen = build_keyed_chain(LARGE);
    frozen.freeze();
    let mut live = build_keyed_chain(LARGE);
    live.make_live();
    for (state, g) in [("frozen", &frozen), ("live", &live)] {
        group.bench_function(BenchmarkId::new("scc", state), |b| {
            b.iter(|| black_box(g.strongly_connected_components().expect("Failed SCC")))
        });
        group.bench_function(BenchmarkId::new("topological_sort", state), |b| {
            b.iter(|| black_box(g.topological_sort().expect("Failed sort")))
        });
        group.bench_function(BenchmarkId::new("shortest_path_len", state), |b| {
            b.iter(|| black_box(g.shortest_path_len(0, LARGE).expect("Failed path")))
        });
    }
    group.finish();
}

// --- Updates in Live: in place and with a rebuild --------------------------------------------

/// A keyed insert into a row of the chain: `in_place` into a row with a spare slot (the row
/// was rebuilt to capacity two by an earlier insert and one entry removed again), `rebuild`
/// into the full row of capacity one every chain node starts with. Both on a fresh clone of
/// the Live chain per batch, so the structure of the measured graph is the same every time.
fn bench_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_update");
    let mut live = build_keyed_chain(LARGE);
    live.make_live();
    let sources = random_indices(1..LARGE - 1);

    // A chain whose node 0 holds a row of capacity two with one spare slot.
    let mut spare = live.clone();
    spare.add_edge(0, 2, Kind(1)).expect("Failed to add edge");
    spare
        .remove_edge_keyed(0, 2, 1)
        .expect("Failed to remove edge");
    group.bench_function("in_place", |b| {
        b.iter_batched_ref(
            || spare.clone(),
            |g| g.add_edge(black_box(0), 2, Kind(1)),
            BatchSize::LargeInput,
        )
    });

    let mut cursor = 0;
    group.bench_function("rebuild", |b| {
        b.iter_batched_ref(
            || {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                (live.clone(), sources[cursor])
            },
            |(g, i)| g.add_edge(black_box(*i), *i + 2, Kind(1)),
            BatchSize::LargeInput,
        )
    });
    group.finish();
}

// --- Transitions -----------------------------------------------------------------------------

/// Every transition into and out of the Live state, and `freeze_clone` from each state, on
/// the chain of 1,000; the source is cloned per batch where the transition consumes it.
fn bench_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("live_transitions");
    let dynamic = build_keyed_chain(LARGE);
    let mut frozen = dynamic.clone();
    frozen.freeze();
    let mut live = dynamic.clone();
    live.make_live();

    group.bench_function("make_live/from_dynamic", |b| {
        b.iter_batched_ref(|| dynamic.clone(), |g| g.make_live(), BatchSize::LargeInput)
    });
    group.bench_function("make_live/from_frozen", |b| {
        b.iter_batched_ref(|| frozen.clone(), |g| g.make_live(), BatchSize::LargeInput)
    });
    group.bench_function("freeze/from_live", |b| {
        b.iter_batched_ref(|| live.clone(), |g| g.freeze(), BatchSize::LargeInput)
    });
    group.bench_function("unfreeze/from_live", |b| {
        b.iter_batched_ref(|| live.clone(), |g| g.unfreeze(), BatchSize::LargeInput)
    });
    for (state, g) in [("dynamic", &dynamic), ("frozen", &frozen), ("live", &live)] {
        group.bench_function(BenchmarkId::new("freeze_clone", state), |b| {
            b.iter_with_large_drop(|| g.freeze_clone())
        });
    }
    group.finish();
}

fn live_benchmarks(c: &mut Criterion) {
    bench_writes_then_scc(c);
    bench_algorithms(c);
    bench_updates(c);
    bench_transitions(c);
}

criterion_group! {
    name = live_bench_collection;
    config = Criterion::default().sample_size(100);
    targets = live_benchmarks
}
