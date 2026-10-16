/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cost a mutation pays for its maintained views: one keyed write and one payload update
//! on the keyed chain of `LARGE` nodes with 0, 1, 4 and 16 views registered, in the Dynamic and
//! the Live state. Every view passes every node and edge, so each write updates each view's
//! rows: the most a view can cost a mutation.

use criterion::{BenchmarkId, Criterion, criterion_group};
use std::hint::black_box;
use ultragraph::*;

use crate::benchmarks::data::{Data, Kind};
use crate::benchmarks::fields::{INDEX_COUNT, LARGE};
use crate::benchmarks::utils::{kind_of, random_indices};

/// The numbers of views the rows register.
const VIEW_COUNTS: [usize; 4] = [0, 1, 4, 16];

/// A projection passing every node and every edge.
#[derive(Clone, Copy, Debug)]
struct Everything;

impl Projection<Data, Kind> for Everything {
    fn node(&self, _node: &Data) -> bool {
        true
    }

    fn edge(&self, _edge: &Kind) -> bool {
        true
    }
}

type Watched = UltraGraphContainer<Data, Kind, Everything>;

/// The keyed chain of `k + 1` nodes with `views` maintained views, dynamic.
fn chain_with_views(k: usize, views: usize) -> Watched {
    let mut g = Watched::with_capacity(k + 1, None);
    for _ in 0..=k {
        g.add_node(Data::default()).expect("Failed to add node");
    }
    for i in 0..k {
        g.add_edge(i, i + 1, kind_of(i))
            .expect("Failed to add edge");
    }
    for _ in 0..views {
        g.maintain(Everything, ParallelEdges::Merge);
    }
    g
}

/// A keyed write (the payload of edge `i → i + 1` written again) on the chain of `LARGE`
/// nodes, with each number of views, dynamic and live.
fn bench_keyed_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("view_upkeep");
    let sources = random_indices(0..LARGE);
    for live in [false, true] {
        for views in VIEW_COUNTS {
            let mut g = chain_with_views(LARGE, views);
            if live {
                g.make_live();
            }
            let state = if live { "live" } else { "dynamic" };
            let mut cursor = 0;
            group.bench_with_input(
                BenchmarkId::new(format!("keyed_write/{state}"), views),
                &views,
                |b, _| {
                    b.iter(|| {
                        cursor = (cursor + 1) & (INDEX_COUNT - 1);
                        let i = sources[cursor];
                        g.add_edge(black_box(i), i + 1, kind_of(i))
                    })
                },
            );
        }
    }
    group.finish();
}

/// A payload update that keeps the node in every view, on the chain of `LARGE` nodes, with
/// each number of views, dynamic and live.
fn bench_update_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("view_upkeep");
    let nodes = random_indices(0..LARGE + 1);
    for live in [false, true] {
        for views in VIEW_COUNTS {
            let mut g = chain_with_views(LARGE, views);
            if live {
                g.make_live();
            }
            let state = if live { "live" } else { "dynamic" };
            let mut cursor = 0;
            group.bench_with_input(
                BenchmarkId::new(format!("update_node/{state}"), views),
                &views,
                |b, _| {
                    b.iter(|| {
                        cursor = (cursor + 1) & (INDEX_COUNT - 1);
                        g.update_node(nodes[cursor], black_box(Data::default()))
                    })
                },
            );
        }
    }
    group.finish();
}

fn view_benchmarks(c: &mut Criterion) {
    bench_keyed_write(c);
    bench_update_node(c);
}

criterion_group! {
    name = view_bench_collection;
    config = Criterion::default().sample_size(100);
    targets = view_benchmarks
}
