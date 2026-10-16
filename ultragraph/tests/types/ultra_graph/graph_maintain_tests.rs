/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Maintained views (spec: ultragraph-maintained-views, and the view scenarios of
//! ultragraph-three-states). The independent source of every expected view is a projection
//! built afresh: `freeze_clone` of the graph, then `project` with the same tests and mode, a
//! view that filters at read time over the CSR and shares no code with the eager maintenance.

use deep_causality_rand::{Rng, Xoshiro256};
use std::fmt::Debug;
use ultragraph::*;

// --- Payloads and projections ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Zone {
    A,
    B,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    #[default]
    A,
    B,
}

/// A payload keyed by its kind and ordered by its weight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Weighted {
    weight: u32,
    kind: Kind,
}

impl EdgeKey for Weighted {
    type Key = Kind;
    fn edge_key(&self) -> Kind {
        self.kind
    }
}

impl core::ops::Add for Weighted {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            weight: self.weight + rhs.weight,
            kind: self.kind,
        }
    }
}

fn weighted(kind: Kind, weight: u32) -> Weighted {
    Weighted { weight, kind }
}

/// A caller's projection type: one variant per scope, every test dispatched statically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    ZoneA,
    ZoneB,
    /// Zone A, heavy edges only.
    HeavyA,
    Everything,
}

impl Projection<Zone, Weighted> for Scope {
    fn node(&self, node: &Zone) -> bool {
        match self {
            Scope::ZoneA | Scope::HeavyA => *node == Zone::A,
            Scope::ZoneB => *node == Zone::B,
            Scope::Everything => true,
        }
    }

    fn edge(&self, edge: &Weighted) -> bool {
        match self {
            Scope::HeavyA => edge.weight >= 5,
            _ => true,
        }
    }
}

type Graph = UltraGraphContainer<Zone, Weighted, Scope>;

/// Nodes 0 and 1 in zone A, 2 in zone B, 3 in zone A, with the edges 0 → 1 and 1 → 2.
fn two_zones() -> Graph {
    let mut g = Graph::new();
    for zone in [Zone::A, Zone::A, Zone::B, Zone::A] {
        g.add_node(zone).unwrap();
    }
    g.add_edge(0, 1, weighted(Kind::A, 1)).unwrap();
    g.add_edge(1, 2, weighted(Kind::A, 1)).unwrap();
    g
}

fn sorted<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    v.sort();
    v
}

/// Every read, traversal and algorithm of a maintained view against a projection view with the
/// same tests and mode over a frozen copy of the graph.
fn assert_view_equals_projection(
    g: &Graph,
    id: ViewId,
    scope: Scope,
    mode: ParallelEdges,
    step: usize,
) {
    let view = g.view(id).expect("the view exists");
    let copy = g.freeze_clone();
    let expected = copy.project(scope, mode).unwrap();
    assert_eq!(view.mode(), mode);
    assert_eq!(
        view.number_nodes(),
        expected.number_nodes(),
        "step {step}: nodes"
    );
    assert_eq!(
        view.number_edges(),
        expected.number_edges(),
        "step {step}: edges"
    );
    assert_eq!(view.is_empty(), expected.is_empty(), "step {step}: empty");
    assert_eq!(
        view.get_last_index(),
        expected.get_last_index(),
        "step {step}: last"
    );
    assert_eq!(
        view.get_root_index(),
        expected.get_root_index(),
        "step {step}: root"
    );
    assert_eq!(
        view.get_root_node(),
        expected.get_root_node(),
        "step {step}: root node"
    );
    assert_eq!(view.contains_root_node(), expected.contains_root_node());
    assert_eq!(
        sorted(view.get_all_nodes()),
        sorted(expected.get_all_nodes()),
        "step {step}: all"
    );
    let bound = g.get_last_index().map_or(1, |last| last + 2);
    for p in 0..bound {
        assert_eq!(
            view.contains_node(p),
            expected.contains_node(p),
            "step {step}: node {p}"
        );
        assert_eq!(
            view.get_node(p),
            expected.get_node(p),
            "step {step}: payload {p}"
        );
        assert_eq!(
            view.get_edges(p),
            expected.get_edges(p),
            "step {step}: edges of {p}"
        );
        assert_eq!(
            view.outbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            expected
                .outbound_edges(p)
                .map(|i| i.collect::<Vec<_>>())
                .ok(),
            "step {step}: out {p}"
        );
        assert_eq!(
            view.inbound_edges(p).map(|i| i.collect::<Vec<_>>()).ok(),
            expected
                .inbound_edges(p)
                .map(|i| i.collect::<Vec<_>>())
                .ok(),
            "step {step}: in {p}"
        );
        for q in 0..bound {
            assert_eq!(
                view.contains_edge(p, q),
                expected.contains_edge(p, q),
                "step {step}: {p} {q}"
            );
            for kind in [Kind::A, Kind::B] {
                assert_eq!(
                    view.contains_edge_keyed(p, q, kind),
                    expected.contains_edge_keyed(p, q, kind),
                    "step {step}: {p} {q} {kind:?}"
                );
            }
            assert_eq!(
                view.is_reachable(p, q),
                expected.is_reachable(p, q),
                "step {step}: reach {p} {q}"
            );
            assert_eq!(
                view.shortest_path(p, q),
                expected.shortest_path(p, q),
                "step {step}: path {p} {q}"
            );
            assert_eq!(
                view.shortest_weighted_path(p, q),
                expected.shortest_weighted_path(p, q),
                "step {step}: weighted {p} {q}"
            );
        }
    }
    assert_eq!(
        view.strongly_connected_components(),
        expected.strongly_connected_components(),
        "step {step}: scc"
    );
    assert_eq!(
        view.articulation_points(),
        expected.articulation_points(),
        "step {step}: ap"
    );
    assert_eq!(view.bridges(), expected.bridges(), "step {step}: bridges");
    assert_eq!(
        view.biconnected_components(),
        expected.biconnected_components(),
        "step {step}: bcc"
    );
    assert_eq!(
        view.find_cycle(),
        expected.find_cycle(),
        "step {step}: cycle"
    );
    assert_eq!(
        view.has_cycle(),
        expected.has_cycle(),
        "step {step}: has_cycle"
    );
    assert_eq!(
        view.topological_sort(),
        expected.topological_sort(),
        "step {step}: topo"
    );
    for directed in [true, false] {
        assert_eq!(
            view.betweenness_centrality(directed, true),
            expected.betweenness_centrality(directed, true),
            "step {step}: betweenness {directed}"
        );
    }
}

// --- Scenarios of ultragraph-maintained-views ------------------------------------------------

/// "Several scopes under one type": views of two variants of the caller's enum on one graph.
#[test]
fn test_several_scopes_under_one_type() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    let b = g.maintain(Scope::ZoneB, ParallelEdges::Merge);
    assert_ne!(a, b);
    assert_eq!(
        sorted(g.view(a).unwrap().get_all_nodes()),
        vec![&Zone::A, &Zone::A, &Zone::A]
    );
    assert_eq!(g.view(b).unwrap().get_all_nodes(), vec![&Zone::B]);
    assert_eq!(g.view(a).unwrap().number_edges(), 1);
    assert_eq!(g.view(b).unwrap().number_edges(), 0);
}

/// "A view answers for its scope": the edge 1 → 2 crosses into zone B, so the zone-A view does
/// not reach 2 while the graph does.
#[test]
fn test_a_view_answers_for_its_scope() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.make_live();
    assert_eq!(g.view(a).unwrap().is_reachable(0, 2), Ok(false));
    assert_eq!(g.is_reachable(0, 2), Ok(true));
    assert_eq!(g.view(a).unwrap().is_reachable(0, 1), Ok(true));
}

/// "A view answers while the graph is dynamic".
#[test]
fn test_a_view_answers_while_the_graph_is_dynamic() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    assert!(!g.is_frozen() && !g.is_live());
    assert_eq!(g.view(a).unwrap().is_reachable(0, 1), Ok(true));
    assert_eq!(g.is_reachable(0, 1), Err(GraphError::GraphNotFrozen));
    // Kahn's order: 0 and 3 start with no incoming edge, in position order, then 1.
    assert_eq!(
        g.view(a).unwrap().topological_sort(),
        Ok(Some(vec![0, 3, 1]))
    );
}

/// "An edge added in scope appears in the view" with no call but the mutation.
#[test]
fn test_an_edge_added_in_scope_appears_in_the_view() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.add_edge(0, 3, weighted(Kind::A, 2)).unwrap();
    assert_eq!(
        g.view(a)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert_eq!(
        g.view(a)
            .unwrap()
            .inbound_edges(3)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![0]
    );
    assert_eq!(g.view(a).unwrap().number_edges(), 2);
    // An edge out of scope does not.
    g.add_edge(2, 3, weighted(Kind::A, 2)).unwrap();
    assert_eq!(
        g.view(a)
            .unwrap()
            .inbound_edges(3)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![0]
    );
    assert_eq!(g.view(a).unwrap().number_edges(), 2);
}

/// "A node leaving the scope leaves the view with its edges": `update_node` into zone B.
#[test]
fn test_a_node_leaving_the_scope_leaves_with_its_edges() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.update_node(1, Zone::B).unwrap();
    let view = g.view(a).unwrap();
    assert!(!view.contains_node(1));
    assert_eq!(view.outbound_edges(0).unwrap().count(), 0);
    assert!(matches!(
        view.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert_eq!(view.number_edges(), 0);
    assert_eq!(view.number_nodes(), 2);
    // Back into zone A, the node and its edges return.
    g.update_node(1, Zone::A).unwrap();
    let view = g.view(a).unwrap();
    assert!(view.contains_node(1));
    assert_eq!(view.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![1]);
    assert_eq!(view.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(view.number_edges(), 1);
}

/// "A view of open relations follows a closing write": the edge test reads the payload, and a
/// replacement with a failing payload takes the edge out of the view.
#[test]
fn test_a_view_follows_a_payload_replacement() {
    let mut g = two_zones();
    g.add_edge(0, 3, weighted(Kind::A, 9)).unwrap();
    let heavy = g.maintain(Scope::HeavyA, ParallelEdges::Merge);
    assert_eq!(
        g.view(heavy)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![3]
    );
    g.add_edge(0, 3, weighted(Kind::A, 1)).unwrap();
    assert_eq!(g.view(heavy).unwrap().outbound_edges(0).unwrap().count(), 0);
    assert_eq!(g.view(heavy).unwrap().number_edges(), 0);
    g.add_edge(0, 1, weighted(Kind::A, 7)).unwrap();
    assert_eq!(
        g.view(heavy)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(
        g.view(heavy)
            .unwrap()
            .inbound_edges(1)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![0]
    );
}

/// "A view after compaction": `compact_ids` rebuilds the view at the new positions, equal to
/// a view newly maintained on the compacted graph.
#[test]
fn test_compact_ids_carries_the_view_through_the_renumbering() {
    let mut g = two_zones();
    g.add_edge(0, 3, weighted(Kind::B, 4)).unwrap();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Distinct);
    g.remove_node(1).unwrap();
    let table = g.compact_ids().unwrap();
    assert_eq!(table, vec![Some(0), None, Some(1), Some(2)]);
    let fresh = g.maintain(Scope::ZoneA, ParallelEdges::Distinct);
    let carried = g.view(a).unwrap();
    let rebuilt = g.view(fresh).unwrap();
    assert_eq!(carried.number_nodes(), 2);
    assert_eq!(carried.get_edges(0), rebuilt.get_edges(0));
    assert_eq!(carried.get_edges(0), Some(vec![(2, &weighted(Kind::B, 4))]));
    assert_eq!(
        carried.inbound_edges(2).unwrap().collect::<Vec<_>>(),
        rebuilt.inbound_edges(2).unwrap().collect::<Vec<_>>()
    );
    assert_eq!(
        carried.strongly_connected_components(),
        rebuilt.strongly_connected_components()
    );
    assert_view_equals_projection(&g, a, Scope::ZoneA, ParallelEdges::Distinct, 0);
}

/// "Views read after freezing in place".
#[test]
fn test_freeze_keeps_the_views() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    let before: Vec<Option<Vec<usize>>> = (0..4)
        .map(|p| {
            g.view(a)
                .unwrap()
                .outbound_edges(p)
                .map(|i| i.collect())
                .ok()
        })
        .collect();
    g.freeze();
    assert!(g.is_frozen());
    let view = g.view(a).unwrap();
    assert!(view.is_frozen());
    let after: Vec<Option<Vec<usize>>> = (0..4)
        .map(|p| view.outbound_edges(p).map(|i| i.collect()).ok())
        .collect();
    assert_eq!(after, before);
    assert_eq!(view.is_reachable(0, 1), Ok(true));
}

// --- The view scenarios of ultragraph-three-states ----------------------------------------------

/// "Views survive every transition": Dynamic → Live → Frozen → Dynamic → Frozen → Live.
#[test]
fn test_views_survive_every_transition() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    let b = g.maintain(Scope::ZoneB, ParallelEdges::Distinct);
    let read = |g: &Graph| -> Vec<String> {
        [a, b]
            .iter()
            .map(|id| {
                let v = g.view(*id).unwrap();
                format!(
                    "{:?} {:?} {:?} {:?}",
                    v.number_nodes(),
                    v.get_edges(0),
                    v.get_edges(1),
                    v.strongly_connected_components()
                )
            })
            .collect()
    };
    let before = read(&g);
    g.make_live();
    assert_eq!(read(&g), before, "Dynamic → Live");
    g.freeze();
    assert_eq!(read(&g), before, "Live → Frozen");
    g.unfreeze();
    assert_eq!(read(&g), before, "Frozen → Dynamic");
    g.freeze();
    assert_eq!(read(&g), before, "Dynamic → Frozen");
    g.make_live();
    assert_eq!(read(&g), before, "Frozen → Live");
    // Still maintained after the round.
    g.add_edge(3, 0, weighted(Kind::A, 1)).unwrap();
    assert_eq!(
        g.view(a)
            .unwrap()
            .inbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![3]
    );
}

/// "The copy carries no maintained view": `freeze_clone` of a Live graph with two views.
#[test]
fn test_freeze_clone_leaves_the_views_behind() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    let b = g.maintain(Scope::ZoneB, ParallelEdges::Merge);
    g.make_live();
    let copy = g.freeze_clone();
    assert!(copy.is_frozen());
    assert!(copy.view(a).is_none(), "the copy has no view");
    g.add_edge(0, 3, weighted(Kind::A, 1)).unwrap();
    assert_eq!(
        g.view(a)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert_eq!(g.view(b).unwrap().number_nodes(), 1);
    assert_eq!(copy.number_edges(), 2);
}

/// "Clearing a Live graph": the view stays registered and holds nothing until a node passes.
#[test]
fn test_clearing_a_live_graph_keeps_the_view_registered_and_empty() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.make_live();
    g.clear().unwrap();
    assert!(!g.is_live());
    assert_eq!(g.add_node(Zone::B), Ok(0));
    let view = g.view(a).expect("still registered");
    assert_eq!(view.number_nodes(), 0);
    assert!(view.is_empty());
    assert!(!view.contains_node(0));
    assert_eq!(g.add_node(Zone::A), Ok(1));
    assert_eq!(g.view(a).unwrap().number_nodes(), 1);
    assert!(g.view(a).unwrap().contains_node(1));
}

#[test]
fn test_an_identifier_the_graph_did_not_issue_reads_as_none() {
    let mut g = two_zones();
    let mut other = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    assert!(g.view(a).is_some());
    assert!(other.view(a).is_none());
    let _ = other.maintain(Scope::ZoneB, ParallelEdges::Merge);
    // The other graph's identifier 0 names its own first view, not this graph's.
    assert_eq!(other.view(a).unwrap().number_nodes(), 1);
}

// --- Rows A, B, C, F and H ---------------------------------------------------------------------

/// Row A: a view maintained on an empty graph, then nodes arriving.
#[test]
fn test_a_view_on_an_empty_graph() {
    let mut g = Graph::new();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    let view = g.view(a).unwrap();
    assert!(view.is_empty());
    assert_eq!(view.number_nodes(), 0);
    assert_eq!(view.number_edges(), 0);
    assert_eq!(view.get_last_index(), None);
    assert_eq!(view.strongly_connected_components(), Ok(vec![]));
    assert_eq!(view.topological_sort(), Ok(Some(vec![])));
    g.add_node(Zone::B).unwrap();
    g.add_node(Zone::A).unwrap();
    assert_eq!(g.view(a).unwrap().number_nodes(), 1);
    assert_eq!(g.view(a).unwrap().get_last_index(), Some(1));
}

/// Row B: a view passing one node, with a self-loop that passes and an edge that does not.
#[test]
fn test_a_view_passing_one_node() {
    let mut g = Graph::new();
    g.add_root_node(Zone::A).unwrap();
    g.add_node(Zone::B).unwrap();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.add_edge(0, 0, weighted(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, weighted(Kind::A, 1)).unwrap();
    let view = g.view(a).unwrap();
    assert_eq!(view.number_nodes(), 1);
    assert_eq!(view.number_edges(), 1);
    assert_eq!(view.get_root_index(), Some(0));
    assert_eq!(view.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(view.has_cycle(), Ok(true));
    assert_eq!(view.find_cycle(), Ok(Some(vec![0, 0])));
}

/// Row C: a view passing everything equals the graph, beside one that does not.
#[test]
fn test_a_view_passing_everything_equals_the_graph() {
    let mut g = two_zones();
    g.add_edge(2, 0, weighted(Kind::B, 3)).unwrap();
    let all = g.maintain(Scope::Everything, ParallelEdges::Merge);
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.make_live();
    let view = g.view(all).unwrap();
    assert_eq!(view.number_nodes(), g.number_nodes());
    assert_eq!(view.number_edges(), g.number_edges());
    for p in 0..4 {
        assert_eq!(view.get_edges(p), g.get_edges(p));
    }
    assert_eq!(
        view.strongly_connected_components(),
        g.strongly_connected_components()
    );
    assert_eq!(view.bridges(), g.bridges());
    assert_ne!(g.view(a).unwrap().number_edges(), g.number_edges());
}

/// Row F: a weight of zero passes an edge test that does not read it, and is routed.
#[test]
fn test_a_zero_weight_in_a_view() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.add_edge(0, 3, weighted(Kind::B, 0)).unwrap();
    let view = g.view(a).unwrap();
    assert!(view.contains_edge_keyed(0, 3, Kind::B));
    // The path's total starts from the default payload and adds the edge's weight, 0.
    let (path, total) = view.shortest_weighted_path(0, 3).unwrap().unwrap();
    assert_eq!(path, vec![0, 3]);
    assert_eq!(total.weight, 0);
}

/// Row H: the test flips on the last node: the last position leaves and returns.
#[test]
fn test_the_view_test_flips_on_the_last_node() {
    let mut g = two_zones();
    let a = g.maintain(Scope::ZoneA, ParallelEdges::Merge);
    g.add_edge(0, 3, weighted(Kind::A, 1)).unwrap();
    assert_eq!(g.view(a).unwrap().get_last_index(), Some(3));
    g.update_node(3, Zone::B).unwrap();
    assert_eq!(g.view(a).unwrap().get_last_index(), Some(1));
    assert_eq!(
        g.view(a)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1]
    );
    g.update_node(3, Zone::A).unwrap();
    assert_eq!(g.view(a).unwrap().get_last_index(), Some(3));
    assert_eq!(
        g.view(a)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    g.remove_node(3).unwrap();
    assert_eq!(g.view(a).unwrap().get_last_index(), Some(1));
    assert_eq!(g.view(a).unwrap().number_edges(), 1);
}

// --- Agreement over generated mutation sequences ------------------------------------------------

/// "Agreement over generated mutation sequences": four maintained views of different tests and
/// modes, every one compared after every step with a projection built afresh from a frozen copy
/// of the current graph. The sequence runs in the Dynamic state and the Live state in turn, with
/// node additions and removals, payload updates across the zones, keyed edge additions,
/// replacements that change the weight, keyed and pair removals, a root, and `compact_ids` in
/// the Dynamic state.
fn drive(seed: u64, steps: usize) {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut g = Graph::new();
    for zone in [Zone::A, Zone::B, Zone::A] {
        g.add_node(zone).unwrap();
    }
    let views = [
        (
            g.maintain(Scope::ZoneA, ParallelEdges::Merge),
            Scope::ZoneA,
            ParallelEdges::Merge,
        ),
        (
            g.maintain(Scope::ZoneB, ParallelEdges::Distinct),
            Scope::ZoneB,
            ParallelEdges::Distinct,
        ),
        (
            g.maintain(Scope::HeavyA, ParallelEdges::Distinct),
            Scope::HeavyA,
            ParallelEdges::Distinct,
        ),
        (
            g.maintain(Scope::Everything, ParallelEdges::Merge),
            Scope::Everything,
            ParallelEdges::Merge,
        ),
    ];
    let zone = |rng: &mut Xoshiro256| {
        if rng.random_range(0..2u8) == 0 {
            Zone::A
        } else {
            Zone::B
        }
    };
    let kind = |rng: &mut Xoshiro256| {
        if rng.random_range(0..2u8) == 0 {
            Kind::A
        } else {
            Kind::B
        }
    };

    for step in 0..steps {
        let bound = g.get_last_index().map_or(0, |last| last + 1) + 1;
        let a = rng.random_range(0..bound);
        let b = rng.random_range(0..bound);
        match rng.random_range(0..100usize) {
            0..=39 => {
                let w = weighted(kind(&mut rng), rng.random_range(0..10u32));
                let _ = g.add_edge(a, b, w);
            }
            40..=49 => {
                let _ = g.remove_edge_keyed(a, b, kind(&mut rng));
            }
            50..=56 => {
                let _ = g.remove_edge(a, b);
            }
            57..=66 => {
                if g.number_nodes() < 8 {
                    g.add_node(zone(&mut rng)).unwrap();
                }
            }
            67..=73 => {
                let _ = g.remove_node(a);
            }
            74..=85 => {
                let _ = g.update_node(a, zone(&mut rng));
            }
            86..=88 => {
                let _ = g.add_root_node(zone(&mut rng));
            }
            89..=92 => {
                if g.is_live() {
                    g.unfreeze();
                } else {
                    g.make_live();
                }
            }
            93..=95 if !g.is_live() => {
                g.compact_ids().unwrap();
            }
            _ => {}
        }
        for (id, scope, mode) in views {
            assert_view_equals_projection(&g, id, scope, mode, step);
        }
    }
}

#[test]
fn test_four_views_agree_with_fresh_projections_over_generated_sequences() {
    for seed in [81, 82, 83] {
        drive(seed, 250);
    }
}
