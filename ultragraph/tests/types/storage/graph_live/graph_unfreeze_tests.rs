/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Live → Dynamic (spec: ultragraph-three-states), through the container.

use ultragraph::*;

/// 0 → 1, 0 → 2, 1 → 2 and 3 → 1 on four nodes, made live; then 3 removed and 2 → 3's would-be
/// replacement 2 → 0 added in Live.
fn live_after_mutations() -> UltraGraph<u8> {
    let mut g = UltraGraph::new();
    for p in 0..4u8 {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(3, 1, ()).unwrap();
    g.make_live();
    g.remove_node(3).unwrap();
    g.add_edge(2, 0, ()).unwrap();
    g
}

#[test]
fn test_unfreeze_from_live_keeps_positions_edges_and_tombstones() {
    let mut g = live_after_mutations();
    g.unfreeze();
    assert!(!g.is_live());
    assert!(!g.is_frozen());
    assert_eq!(g.number_nodes(), 3);
    assert!(!g.contains_node(3));
    assert_eq!(g.get_last_index(), Some(2));
    // The stale 3 → 1 left; the Live edge 2 → 0 stayed: four edges.
    assert_eq!(g.number_edges(), 4);
    assert_eq!(g.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(g.inbound_edges(0).unwrap().collect::<Vec<_>>(), vec![2]);
    // Mutable again, and the tombstone is not reissued.
    assert_eq!(g.add_node(9), Ok(4));
    assert_eq!(g.add_edge(4, 1, ()), Ok(()));
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0, 4]);
}

#[test]
fn test_unfreeze_of_an_empty_live_graph() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    g.make_live();
    g.unfreeze();
    assert!(!g.is_live());
    assert!(g.is_empty());
    assert_eq!(g.add_node(1), Ok(0));
}

/// Live → Dynamic → Live: the second adoption reads as the first.
#[test]
fn test_live_dynamic_live_round_trip() {
    let mut g = live_after_mutations();
    let edges_before: Vec<Option<Vec<(usize, ())>>> = (0..4)
        .map(|p| {
            g.get_edges(p)
                .map(|e| e.into_iter().map(|(t, w)| (t, *w)).collect())
        })
        .collect();
    g.unfreeze();
    g.make_live();
    assert!(g.is_live());
    let edges_after: Vec<Option<Vec<(usize, ())>>> = (0..4)
        .map(|p| {
            g.get_edges(p)
                .map(|e| e.into_iter().map(|(t, w)| (t, *w)).collect())
        })
        .collect();
    assert_eq!(edges_after, edges_before);
    assert_eq!(g.number_edges(), 4);
}
