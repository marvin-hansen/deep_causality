/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The workloads the edge-storage decision of `ultragraph-live-state-maintained-views` rests on:
//! keyed update and removal on low-degree nodes and on a hub, traversal in the dynamic state,
//! and memory per edge.
//!
//! Fixtures:
//! - keyed chain (`utils::build_keyed_chain(n)`): `n + 1` nodes, edge `i → i + 1` of kind
//!   `kind_of(i)`, every node of degree at most 1.
//! - keyed hub (`utils::build_keyed_hub(HUB_LEAVES)`): a hub with `KINDS` edges to each of
//!   `HUB_LEAVES` leaves, one per kind: `HUB_EDGES` edges out of one node.
//! - chain and shuffled chain, as in `bench_crud`, left in the dynamic state.
//!
//! An update replaces the payload of an edge that exists, `(target, key)` found by the
//! storage's lookup; a removal takes one keyed edge out. A removal changes the graph, so its
//! benchmark builds a fresh fixture per batch; the hub is cloned rather than rebuilt, since
//! building it by inserts scans the hub's list once per edge.
//!
//! `memory_per_edge` is not a timing: it prints the deterministic byte count of each fixture,
//! before and after the update workload, so the run log carries it; the measured call is the
//! accounting itself.

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group};
use deep_causality_rand::Rng;
use std::hint::black_box;
use ultragraph::utils::utils_tests::{edge_storage_bytes, memory_per_edge};
use ultragraph::*;

use crate::benchmarks::data::{Data, Kind};
use crate::benchmarks::fields::{INDEX_COUNT, LARGE, SIZES};
use crate::benchmarks::utils::{
    KINDS, build_keyed_chain, build_keyed_hub, build_linear_graph, build_shuffled_chain, kind_of,
    random_indices, seeded_rng, walk,
};

const HUB: usize = 0;
/// Leaves of the keyed hub; with `KINDS` edges each the hub carries `HUB_EDGES` edges.
const HUB_LEAVES: usize = 25_000;
const HUB_EDGES: usize = HUB_LEAVES * KINDS as usize;
/// Samples for the hub removals, whose per-batch clone copies `HUB_EDGES` edges.
const HUB_SAMPLES: usize = 20;

// --- Keyed update -------------------------------------------------------------------------

/// Replaces the payload of a random edge of a keyed chain of `n + 1` nodes.
fn bench_keyed_update_low(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyed_update_low");
    for n in SIZES {
        let mut g = build_keyed_chain(n);
        let sources = random_indices(0..n);
        let mut cursor = 0;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                let i = sources[cursor];
                g.add_edge(black_box(i), i + 1, kind_of(i))
            })
        });
    }
    group.finish();
}

/// Replaces the payload of a random keyed edge of the hub.
fn bench_keyed_update_hub(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyed_update_hub");
    let mut g = build_keyed_hub(HUB_LEAVES);
    let leaves = random_indices(1..HUB_LEAVES + 1);
    let kinds = random_indices(0..KINDS as usize);
    let mut cursor = 0;
    group.bench_with_input(
        BenchmarkId::from_parameter(HUB_EDGES),
        &HUB_EDGES,
        |b, _| {
            b.iter(|| {
                cursor = (cursor + 1) & (INDEX_COUNT - 1);
                let kind = Kind(kinds[cursor] as u8);
                g.add_edge(black_box(HUB), leaves[cursor], kind)
            })
        },
    );
    group.finish();
}

// --- Keyed removal ------------------------------------------------------------------------

/// Removes a random keyed edge of a keyed chain of `n + 1` nodes.
fn bench_keyed_remove_low(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyed_remove_low");
    for n in SIZES {
        let mut rng = seeded_rng();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched_ref(
                || (build_keyed_chain(n), rng.random_range(0..n)),
                |(g, i)| g.remove_edge_keyed(black_box(*i), *i + 1, kind_of(*i).0),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

/// Removes a random keyed edge of the hub, from a clone of the hub per batch.
fn bench_keyed_remove_hub(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyed_remove_hub");
    group.sample_size(HUB_SAMPLES);
    let hub = build_keyed_hub(HUB_LEAVES);
    let mut rng = seeded_rng();
    group.bench_with_input(
        BenchmarkId::from_parameter(HUB_EDGES),
        &HUB_EDGES,
        |b, _| {
            b.iter_batched_ref(
                || {
                    (
                        hub.clone(),
                        rng.random_range(1..HUB_LEAVES + 1),
                        rng.random_range(0..KINDS as usize) as u8,
                    )
                },
                |(g, leaf, kind)| g.remove_edge_keyed(black_box(HUB), *leaf, *kind),
                BatchSize::LargeInput,
            )
        },
    );
    group.finish();
}

// --- Traversal in the dynamic state ---------------------------------------------------------

/// Walks a dynamic chain of `n + 1` nodes from the root to its end through `outbound_edges`.
fn bench_outbound_walk_dynamic(c: &mut Criterion) {
    let mut group = c.benchmark_group("outbound_walk_dynamic");
    for n in SIZES {
        let g = build_linear_graph(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| walk(&g, 0, false))
        });
    }
    group.finish();
}

/// Walks a dynamic chain of `n + 1` nodes from its end back to the root through `inbound_edges`.
fn bench_inbound_walk_dynamic(c: &mut Criterion) {
    let mut group = c.benchmark_group("inbound_walk_dynamic");
    for n in SIZES {
        let g = build_linear_graph(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| walk(&g, n, true))
        });
    }
    group.finish();
}

/// Walks a dynamic shuffled chain of `n + 1` nodes from its first node to its last.
fn bench_outbound_walk_shuffled_dynamic(c: &mut Criterion) {
    let mut group = c.benchmark_group("outbound_walk_shuffled_dynamic");
    for n in SIZES {
        let (g, first, _) = build_shuffled_chain(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| walk(&g, first, false))
        });
    }
    group.finish();
}

/// Walks a dynamic shuffled chain of `n + 1` nodes from its last node back to its first.
fn bench_inbound_walk_shuffled_dynamic(c: &mut Criterion) {
    let mut group = c.benchmark_group("inbound_walk_shuffled_dynamic");
    for n in SIZES {
        let (g, _, last) = build_shuffled_chain(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| walk(&g, last, true))
        });
    }
    group.finish();
}

// --- Memory per edge ----------------------------------------------------------------------

/// Prints the byte count of `g` under `label` and times the accounting call.
fn report_memory(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    label: &str,
    g: &UltraGraphWeighted<Data, Kind>,
) {
    let bytes = edge_storage_bytes(g);
    let edges = g.number_edges();
    let per_edge = memory_per_edge(g).unwrap_or(f64::NAN);
    eprintln!(
        "memory_per_edge/{label}: {bytes} bytes over {edges} edges = {per_edge:.3} bytes per edge"
    );
    group.bench_function(label, |b| b.iter(|| edge_storage_bytes(black_box(g))));
}

/// The deterministic memory per edge of the keyed hub and the keyed chain of `LARGE` nodes,
/// after building and after the keyed update workload.
fn bench_memory_per_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_edge");
    group.sample_size(10);

    let mut hub = build_keyed_hub(HUB_LEAVES);
    report_memory(&mut group, "hub_built", &hub);
    let leaves = random_indices(1..HUB_LEAVES + 1);
    let kinds = random_indices(0..KINDS as usize);
    for (&leaf, &kind) in leaves.iter().zip(&kinds) {
        hub.add_edge(HUB, leaf, Kind(kind as u8))
            .expect("Failed to update edge");
    }
    report_memory(&mut group, "hub_updated", &hub);

    let mut chain = build_keyed_chain(LARGE);
    report_memory(&mut group, "chain_1000_built", &chain);
    let sources = random_indices(0..LARGE);
    for &i in &sources {
        chain
            .add_edge(i, i + 1, kind_of(i))
            .expect("Failed to update edge");
    }
    report_memory(&mut group, "chain_1000_updated", &chain);

    group.finish();
}

fn storage_benchmarks(c: &mut Criterion) {
    bench_keyed_update_low(c);
    bench_keyed_update_hub(c);
    bench_keyed_remove_low(c);
    bench_keyed_remove_hub(c);

    bench_outbound_walk_dynamic(c);
    bench_inbound_walk_dynamic(c);
    bench_outbound_walk_shuffled_dynamic(c);
    bench_inbound_walk_shuffled_dynamic(c);

    bench_memory_per_edge(c);
}

criterion_group! {
    name = storage_bench_collection;
    config = Criterion::default().sample_size(100);
    targets = storage_benchmarks
}
