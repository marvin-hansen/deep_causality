/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reads in the Live state (spec: ultragraph-three-states), through the container.

use ultragraph::*;

/// A chain 0 → 1 → 2 → 3 with a tombstone at 4 and a root at 0, made live from the dynamic
/// state.
fn live_chain() -> UltraGraphWeighted<&'static str, u64> {
    let mut g = UltraGraphWeighted::new();
    assert_eq!(g.add_root_node("a").unwrap(), 0);
    for (p, name) in ["b", "c", "d", "gone"].iter().enumerate() {
        assert_eq!(g.add_node(name).unwrap(), p + 1);
    }
    g.add_edge(0, 1, 10).unwrap();
    g.add_edge(1, 2, 20).unwrap();
    g.add_edge(2, 3, 30).unwrap();
    g.add_edge(3, 4, 40).unwrap();
    g.remove_node(4).unwrap();
    g.make_live();
    g
}

#[test]
fn test_state_queries_in_live() {
    let g = live_chain();
    assert!(g.is_live());
    assert!(!g.is_frozen());
    assert!(!g.is_empty());
}

#[test]
fn test_node_reads_in_live() {
    let g = live_chain();
    assert_eq!(g.number_nodes(), 4);
    assert!(g.contains_node(3));
    assert!(!g.contains_node(4));
    assert!(!g.contains_node(5));
    assert_eq!(g.get_node(2), Some(&"c"));
    assert_eq!(g.get_node(4), None);
    assert_eq!(g.get_last_index(), Some(3));
    let mut all: Vec<&str> = g.get_all_nodes().into_iter().copied().collect();
    all.sort_unstable();
    assert_eq!(all, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_root_reads_in_live() {
    let g = live_chain();
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(0));
    assert_eq!(g.get_root_node(), Some(&"a"));
}

#[test]
fn test_edge_reads_in_live() {
    let g = live_chain();
    // The edge into the tombstone left the count.
    assert_eq!(g.number_edges(), 3);
    assert!(g.contains_edge(1, 2));
    assert!(!g.contains_edge(2, 1));
    assert!(!g.contains_edge(3, 4));
    assert!(!g.contains_edge(4, 0));
    assert!(g.contains_edge_keyed(1, 2, ()));
    assert!(!g.contains_edge_keyed(2, 1, ()));
    assert_eq!(g.get_edges(1), Some(vec![(2, &20)]));
    assert_eq!(g.get_edges(3), Some(vec![]));
    assert_eq!(g.get_edges(4), None);
    assert_eq!(g.get_edges(9), None);
}

/// A wide row read by the binary search the Live state uses for `contains_edge`: every
/// target of a hub of 70 is found and every absent one is not, with a tombstone among the
/// targets whose entry the search skips.
#[test]
fn test_contains_edge_on_a_wide_live_row() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    for p in 0..71 {
        assert_eq!(g.add_node(p as u8).unwrap(), p);
    }
    for t in (1..71).step_by(2) {
        g.add_edge(0, t, ()).unwrap();
    }
    g.make_live();
    g.remove_node(35).unwrap();
    for t in 1..71 {
        let expected = t % 2 == 1 && t != 35;
        assert_eq!(g.contains_edge(0, t), expected, "target {t}");
        assert_eq!(g.contains_edge_keyed(0, t, ()), expected, "target {t}");
    }
    assert_eq!(g.number_edges(), 34);
    assert_eq!(g.outbound_edges(0).unwrap().count(), 34);
}
