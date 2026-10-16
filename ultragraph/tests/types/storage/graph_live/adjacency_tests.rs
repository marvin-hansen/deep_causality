/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algorithms in the Live state (spec: ultragraph-three-states, "Live answers as a freshly
//! frozen graph would"). The independent source is a second graph that receives the same
//! operations in the Dynamic state and is cloned and frozen before every comparison: the Vec
//! lists and the CSR against the arenas, with no shared code below the algorithm bodies.

use deep_causality_rand::{Rng, Xoshiro256};
use std::fmt::Debug;
use ultragraph::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Rel {
    #[default]
    Parent,
    Child,
    Peer,
}

impl EdgeKey for Rel {
    type Key = Rel;
    fn edge_key(&self) -> Rel {
        *self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Tagged {
    w: u32,
    kind: u8,
}

impl EdgeKey for Tagged {
    type Key = u8;
    fn edge_key(&self) -> u8 {
        self.kind
    }
}

impl core::ops::Add for Tagged {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            w: self.w + rhs.w,
            kind: self.kind,
        }
    }
}

fn tagged(kind: u8, w: u32) -> Tagged {
    Tagged { w, kind }
}

/// The weighted shortest path needs an ordered, addable payload; the payloads that have one
/// compare it, the others compare nothing.
trait WeightedCheck: Clone + Default + EdgeKey {
    fn check_weighted(
        live: &UltraGraphWeighted<u8, Self>,
        frozen: &UltraGraphWeighted<u8, Self>,
        p: usize,
        q: usize,
        step: usize,
    ) {
        let _ = (live, frozen, p, q, step);
    }
}

impl WeightedCheck for () {}
impl WeightedCheck for Rel {}

impl WeightedCheck for u64 {
    fn check_weighted(
        live: &UltraGraphWeighted<u8, Self>,
        frozen: &UltraGraphWeighted<u8, Self>,
        p: usize,
        q: usize,
        step: usize,
    ) {
        assert_eq!(
            live.shortest_weighted_path(p, q),
            frozen.shortest_weighted_path(p, q),
            "step {step}: weighted {p} {q}"
        );
    }
}

impl WeightedCheck for Tagged {
    fn check_weighted(
        live: &UltraGraphWeighted<u8, Self>,
        frozen: &UltraGraphWeighted<u8, Self>,
        p: usize,
        q: usize,
        step: usize,
    ) {
        assert_eq!(
            live.shortest_weighted_path(p, q),
            frozen.shortest_weighted_path(p, q),
            "step {step}: weighted {p} {q}"
        );
    }
}

/// Scenario "Live accepts a write and answers an algorithm": after `add_edge(2, 3, w)` on a
/// Live graph, `is_reachable(0, 3)` reflects the new edge without any transition.
#[test]
fn test_a_write_in_live_is_seen_by_the_next_algorithm() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    for p in 0..4u8 {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.make_live();
    assert_eq!(g.is_reachable(0, 3), Ok(false));
    assert_eq!(g.add_edge(2, 3, ()), Ok(()));
    assert!(g.is_live());
    assert_eq!(g.is_reachable(0, 3), Ok(true));
    assert_eq!(g.shortest_path(0, 3), Ok(Some(vec![0, 1, 2, 3])));
    assert_eq!(g.topological_sort(), Ok(Some(vec![0, 1, 2, 3])));
    g.add_edge(3, 0, ()).unwrap();
    assert_eq!(g.has_cycle(), Ok(true));
    assert_eq!(g.strongly_connected_components().map(|c| c.len()), Ok(1));
}

/// Every algorithm on an empty Live graph (row A).
#[test]
fn test_algorithms_on_an_empty_live_graph() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    g.make_live();
    assert_eq!(g.strongly_connected_components(), Ok(vec![]));
    assert_eq!(g.articulation_points(), Ok(vec![]));
    assert_eq!(g.bridges(), Ok(vec![]));
    assert_eq!(g.biconnected_components(), Ok(vec![]));
    assert_eq!(g.find_cycle(), Ok(None));
    assert_eq!(g.has_cycle(), Ok(false));
    assert_eq!(g.topological_sort(), Ok(Some(vec![])));
    assert_eq!(g.is_reachable(0, 0), Ok(false));
    assert_eq!(g.betweenness_centrality(true, true), Ok(vec![]));
}

/// The dynamic state still refuses the algorithms, so the Live answer is not the Dynamic one.
#[test]
fn test_the_dynamic_state_still_refuses_algorithms() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    g.add_node(0).unwrap();
    g.make_live();
    g.unfreeze();
    assert_eq!(g.is_reachable(0, 0), Err(GraphError::GraphNotFrozen));
    assert_eq!(
        g.strongly_connected_components(),
        Err(GraphError::GraphNotFrozen)
    );
}

// --- Agreement with a freshly frozen graph ---------------------------------------------------

fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}

fn sorted_groups(groups: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    sorted(groups.into_iter().map(sorted).collect())
}

/// Every algorithm, read and traversal on `live` against the same on `frozen`.
fn assert_agree<W>(
    live: &UltraGraphWeighted<u8, W>,
    frozen: &UltraGraphWeighted<u8, W>,
    step: usize,
) where
    W: WeightedCheck + Copy + PartialEq + Debug,
    W::Key: Debug,
{
    assert!(live.is_live() && frozen.is_frozen());
    assert_eq!(
        live.number_nodes(),
        frozen.number_nodes(),
        "step {step}: number_nodes"
    );
    assert_eq!(
        live.number_edges(),
        frozen.number_edges(),
        "step {step}: number_edges"
    );
    assert_eq!(
        live.get_last_index(),
        frozen.get_last_index(),
        "step {step}: last"
    );
    assert_eq!(
        live.get_root_index(),
        frozen.get_root_index(),
        "step {step}: root"
    );
    let bound = frozen.get_last_index().map_or(1, |last| last + 2);
    for p in 0..bound {
        assert_eq!(
            live.contains_node(p),
            frozen.contains_node(p),
            "step {step}: node {p}"
        );
        assert_eq!(
            live.get_node(p),
            frozen.get_node(p),
            "step {step}: payload {p}"
        );
        assert_eq!(
            live.get_edges(p),
            frozen.get_edges(p),
            "step {step}: get_edges({p})"
        );
        assert_eq!(
            live.outbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            frozen.outbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            "step {step}: outbound({p})"
        );
        assert_eq!(
            live.inbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            frozen.inbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            "step {step}: inbound({p})"
        );
        for q in 0..bound {
            assert_eq!(
                live.contains_edge(p, q),
                frozen.contains_edge(p, q),
                "step {step}: edge {p} {q}"
            );
            assert_eq!(
                live.is_reachable(p, q),
                frozen.is_reachable(p, q),
                "step {step}: reach {p} {q}"
            );
            assert_eq!(
                live.shortest_path_len(p, q),
                frozen.shortest_path_len(p, q),
                "step {step}: len {p} {q}"
            );
            assert_eq!(
                live.shortest_path(p, q),
                frozen.shortest_path(p, q),
                "step {step}: path {p} {q}"
            );
            W::check_weighted(live, frozen, p, q, step);
        }
    }
    assert_eq!(
        live.strongly_connected_components().map(sorted_groups),
        frozen.strongly_connected_components().map(sorted_groups),
        "step {step}: scc"
    );
    assert_eq!(
        live.articulation_points().map(sorted),
        frozen.articulation_points().map(sorted),
        "step {step}: ap"
    );
    assert_eq!(
        live.bridges().map(sorted),
        frozen.bridges().map(sorted),
        "step {step}: bridges"
    );
    assert_eq!(
        live.biconnected_components().map(sorted_groups),
        frozen.biconnected_components().map(sorted_groups),
        "step {step}: bcc"
    );
    assert_eq!(live.find_cycle(), frozen.find_cycle(), "step {step}: cycle");
    assert_eq!(
        live.has_cycle(),
        frozen.has_cycle(),
        "step {step}: has_cycle"
    );
    assert_eq!(
        live.topological_sort(),
        frozen.topological_sort(),
        "step {step}: topo"
    );
    for directed in [true, false] {
        for normalized in [true, false] {
            assert_eq!(
                live.betweenness_centrality(directed, normalized)
                    .map(sorted_scores),
                frozen
                    .betweenness_centrality(directed, normalized)
                    .map(sorted_scores),
                "step {step}: betweenness {directed} {normalized}"
            );
        }
    }
    let pathways = [(0, bound - 1), (1, 2), (bound / 2, 0)];
    assert_eq!(
        live.pathway_betweenness_centrality(&pathways, true, true)
            .map(sorted_scores),
        frozen
            .pathway_betweenness_centrality(&pathways, true, true)
            .map(sorted_scores),
        "step {step}: pathway betweenness"
    );
}

fn sorted_scores(mut scores: Vec<(usize, f64)>) -> Vec<(usize, u64)> {
    scores.sort_by_key(|(p, _)| *p);
    scores.into_iter().map(|(p, s)| (p, s.to_bits())).collect()
}

/// Random mutations applied to a Live graph and to a Dynamic reference alike, the reference
/// cloned and frozen before every comparison: node additions and removals, keyed edge
/// additions (replacements included), keyed and pair removals, payload updates and a root. On
/// up to eight positions, a node's row outgrows its capacity many times over 300 steps.
fn drive<W, F>(seed: u64, steps: usize, mut draw: F)
where
    W: WeightedCheck + Copy + PartialEq + Debug,
    W::Key: Debug,
    F: FnMut(&mut Xoshiro256) -> W,
{
    let mut rng = Xoshiro256::from_seed(seed);
    let mut live: UltraGraphWeighted<u8, W> = UltraGraphWeighted::new();
    let mut reference: UltraGraphWeighted<u8, W> = UltraGraphWeighted::new();
    for p in 0..3u8 {
        live.add_node(p).unwrap();
        reference.add_node(p).unwrap();
    }
    live.make_live();
    let mut next_payload = 3u8;

    for step in 0..steps {
        let bound = reference.get_last_index().map_or(0, |last| last + 1) + 1;
        let a = rng.random_range(0..bound);
        let b = rng.random_range(0..bound);
        let roll = rng.random_range(0..100usize);
        match roll {
            0..=49 => {
                let w = draw(&mut rng);
                assert_eq!(
                    live.add_edge(a, b, w),
                    reference.add_edge(a, b, w),
                    "step {step}: add_edge({a}, {b})"
                );
            }
            50..=61 => {
                let w = draw(&mut rng);
                assert_eq!(
                    live.remove_edge_keyed(a, b, w.edge_key()),
                    reference.remove_edge_keyed(a, b, w.edge_key()),
                    "step {step}: remove_edge_keyed({a}, {b})"
                );
            }
            62..=69 => {
                assert_eq!(
                    live.remove_edge(a, b),
                    reference.remove_edge(a, b),
                    "step {step}: remove_edge({a}, {b})"
                );
            }
            70..=79 => {
                if reference.number_nodes() < 8 {
                    assert_eq!(
                        live.add_node(next_payload),
                        reference.add_node(next_payload),
                        "step {step}: add_node"
                    );
                    next_payload = next_payload.wrapping_add(1);
                }
            }
            80..=86 => {
                assert_eq!(
                    live.remove_node(a),
                    reference.remove_node(a),
                    "step {step}: remove_node({a})"
                );
            }
            87..=93 => {
                assert_eq!(
                    live.update_node(a, next_payload),
                    reference.update_node(a, next_payload),
                    "step {step}: update"
                );
                next_payload = next_payload.wrapping_add(1);
            }
            94..=96 => {
                assert_eq!(
                    live.add_root_node(next_payload),
                    reference.add_root_node(next_payload),
                    "step {step}: root"
                );
                next_payload = next_payload.wrapping_add(1);
            }
            _ => {}
        }
        let mut frozen = reference.clone();
        frozen.freeze();
        assert_agree(&live, &frozen, step);
    }
}

#[test]
fn test_live_agrees_with_a_freshly_frozen_graph_unit_payloads() {
    for seed in [41, 42] {
        drive::<(), _>(seed, 300, |_| ());
    }
}

#[test]
fn test_live_agrees_with_a_freshly_frozen_graph_weights() {
    for seed in [51, 52] {
        // Weight 0 included.
        drive::<u64, _>(seed, 300, |rng| rng.random_range(0..9u64));
    }
}

#[test]
fn test_live_agrees_with_a_freshly_frozen_graph_identity_keys() {
    let kinds = [Rel::Parent, Rel::Child, Rel::Peer];
    for seed in [61, 62] {
        drive::<Rel, _>(seed, 300, |rng| kinds[rng.random_range(0..3usize)]);
    }
}

#[test]
fn test_live_agrees_with_a_freshly_frozen_graph_field_keys() {
    for seed in [71, 72] {
        drive::<Tagged, _>(seed, 300, |rng| {
            tagged(rng.random_range(0..3u8), rng.random_range(0..20u32))
        });
    }
}
