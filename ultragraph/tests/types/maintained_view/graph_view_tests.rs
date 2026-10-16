/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reads on a maintained view: the passing nodes and edges, the payloads and the root from the
//! graph, in every state.

use ultragraph::*;

/// Odd payloads pass; every edge passes.
type OddNodes = (fn(&u8) -> bool, fn(&u64) -> bool);

fn odd() -> OddNodes {
    (|n| n % 2 == 1, |_| true)
}

/// Payloads 1, 2, 3, 4, 5 at positions 0..5, the root at 0, and position 4 removed; the odd
/// payloads at positions 0 and 2 pass. Edges 0 → 2, 2 → 1, 0 → 1, 1 → 3.
fn graph() -> UltraGraphContainer<u8, u64, OddNodes> {
    let mut g = UltraGraphContainer::new();
    g.add_root_node(1).unwrap();
    for p in [2, 3, 4, 5] {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 2, 10).unwrap();
    g.add_edge(2, 1, 20).unwrap();
    g.add_edge(0, 1, 30).unwrap();
    g.add_edge(1, 3, 40).unwrap();
    g.remove_node(4).unwrap();
    g
}

#[test]
fn test_node_reads_on_a_view() {
    let mut g = graph();
    let id = g.maintain(odd(), ParallelEdges::Merge);
    let view = g.view(id).unwrap();
    // Positions 0 (payload 1) and 2 (payload 3) hold odd payloads; 4 held 5 and is removed.
    assert_eq!(view.number_nodes(), 2);
    assert!(!view.is_empty());
    assert!(view.contains_node(0) && view.contains_node(2));
    assert!(!view.contains_node(1) && !view.contains_node(3));
    assert!(!view.contains_node(4));
    assert!(!view.contains_node(9));
    assert_eq!(view.get_node(2), Some(&3));
    assert_eq!(view.get_node(1), None);
    assert_eq!(view.get_last_index(), Some(2));
    let mut all: Vec<u8> = view.get_all_nodes().into_iter().copied().collect();
    all.sort_unstable();
    assert_eq!(all, vec![1, 3]);
}

#[test]
fn test_root_reads_on_a_view() {
    let mut g = graph();
    let id = g.maintain(odd(), ParallelEdges::Merge);
    assert_eq!(g.view(id).unwrap().get_root_index(), Some(0));
    assert_eq!(g.view(id).unwrap().get_root_node(), Some(&1));
    assert!(g.view(id).unwrap().contains_root_node());
    g.update_node(0, 2).unwrap();
    assert_eq!(g.view(id).unwrap().get_root_index(), None);
    assert!(!g.view(id).unwrap().contains_root_node());
}

#[test]
fn test_edge_reads_on_a_view() {
    let mut g = graph();
    let id = g.maintain(odd(), ParallelEdges::Merge);
    let view = g.view(id).unwrap();
    // 0 → 2 joins two odd nodes; 0 → 1, 2 → 1 and 1 → 3 touch the even node 1.
    assert_eq!(view.number_edges(), 1);
    assert!(view.contains_edge(0, 2));
    assert!(!view.contains_edge(0, 1));
    assert!(!view.contains_edge(2, 1));
    assert!(view.contains_edge_keyed(0, 2, ()));
    assert_eq!(view.get_edges(0), Some(vec![(2, &10)]));
    assert_eq!(view.get_edges(2), Some(vec![]));
    assert_eq!(view.get_edges(1), None);
    assert_eq!(view.get_edges(4), None);
}

#[test]
fn test_the_views_state_is_the_graphs() {
    let mut g = graph();
    let id = g.maintain(odd(), ParallelEdges::Merge);
    assert!(!g.view(id).unwrap().is_frozen());
    g.freeze();
    assert!(g.view(id).unwrap().is_frozen());
    assert_eq!(g.view(id).unwrap().number_edges(), 1);
    g.make_live();
    assert!(!g.view(id).unwrap().is_frozen());
    assert_eq!(g.view(id).unwrap().get_edges(0), Some(vec![(2, &10)]));
}
