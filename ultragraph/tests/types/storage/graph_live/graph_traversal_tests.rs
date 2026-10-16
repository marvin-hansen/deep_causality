/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traversal in the Live state (spec: ultragraph-three-states), through the container.

use ultragraph::*;

/// 0 → 1, 0 → 2, 1 → 2, 2 → 0, 3 → 2, with 3 removed after the edges were added, made live.
fn live_graph() -> UltraGraph<u8> {
    let mut g = UltraGraph::new();
    for p in 0..4u8 {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();
    g.add_edge(3, 2, ()).unwrap();
    g.make_live();
    g
}

#[test]
fn test_outbound_edges_in_live_are_sorted_by_target() {
    let g = live_graph();
    assert_eq!(g.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(g.outbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0]);
    // The mask filter bounds the hint from above by the row's length.
    assert_eq!(g.outbound_edges(1).unwrap().size_hint().1, Some(1));
}

#[test]
fn test_inbound_edges_in_live_are_sorted_by_source() {
    let g = live_graph();
    assert_eq!(
        g.inbound_edges(2).unwrap().collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert_eq!(g.inbound_edges(0).unwrap().collect::<Vec<_>>(), vec![2]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
}

/// A removed source's entries stay in the target's row and every traversal skips them.
#[test]
fn test_traversal_skips_a_removed_neighbour() {
    let mut g = live_graph();
    g.remove_node(3).unwrap();
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0, 1]);
    assert!(matches!(
        g.outbound_edges(3),
        Err(GraphError::NodeNotFound(3))
    ));
    assert!(matches!(
        g.inbound_edges(3),
        Err(GraphError::NodeNotFound(3))
    ));
    g.remove_node(0).unwrap();
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![1]);
    assert_eq!(g.outbound_edges(2).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(1).unwrap().count(), 0);
}

#[test]
fn test_traversal_of_an_absent_position_is_refused_in_live() {
    let g = live_graph();
    assert!(matches!(
        g.outbound_edges(4),
        Err(GraphError::NodeNotFound(4))
    ));
    assert!(matches!(
        g.inbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}

/// Edges added in the Live state are traversed at once, in sorted order among the adopted ones.
#[test]
fn test_edges_added_in_live_join_the_traversal() {
    let mut g = live_graph();
    g.add_edge(2, 1, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();
    assert_eq!(
        g.outbound_edges(2).unwrap().collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0, 2]);
    assert_eq!(g.inbound_edges(3).unwrap().collect::<Vec<_>>(), vec![2]);
}
