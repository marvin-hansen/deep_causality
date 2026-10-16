/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Mutations in the Live state (spec: ultragraph-three-states), through the container. Every
//! expectation is written out by hand from the operations applied.

use ultragraph::*;

/// A payload keyed by one field, with data beside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Tagged {
    kind: u8,
    w: u32,
}

impl EdgeKey for Tagged {
    type Key = u8;
    fn edge_key(&self) -> u8 {
        self.kind
    }
}

fn tagged(kind: u8, w: u32) -> Tagged {
    Tagged { kind, w }
}

/// `n` nodes at positions `0..n`, payload equal to the position, made live from the dynamic
/// state (through a freeze).
fn live_nodes<W: Clone + Default + EdgeKey>(n: usize) -> UltraGraphWeighted<u8, W> {
    let mut g = UltraGraphWeighted::new();
    for p in 0..n {
        assert_eq!(g.add_node(p as u8).unwrap(), p);
    }
    g.make_live();
    assert!(g.is_live());
    g
}

/// The `(target, key)` pairs of `get_edges(source)` in the order listed.
fn targets_and_keys(g: &UltraGraphWeighted<u8, Tagged>, source: usize) -> Vec<(usize, u8)> {
    g.get_edges(source)
        .unwrap()
        .into_iter()
        .map(|(t, w)| (t, w.kind))
        .collect()
}

// --- Row A: the empty graph ----------------------------------------------------------------

#[test]
fn test_an_empty_graph_made_live_is_empty_and_mutable() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    g.make_live();
    assert!(g.is_live());
    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.add_node(7), Ok(0));
    assert_eq!(g.get_node(0), Some(&7));
    assert!(g.is_live());
}

#[test]
fn test_an_empty_frozen_graph_made_live_is_empty() {
    let mut g: UltraGraph<u8> = UltraGraph::new();
    g.freeze();
    g.make_live();
    assert!(g.is_live());
    assert!(g.is_empty());
    assert_eq!(g.add_node(1), Ok(0));
}

// --- Row B: one node, a row of capacity one ------------------------------------------------

/// One node made live carries no edge; its first edge, a self-loop, gives its rows a capacity
/// of one and is read back from both sides.
#[test]
fn test_one_node_takes_a_self_loop_in_a_row_of_capacity_one() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(1);
    assert_eq!(g.add_edge(0, 0, tagged(1, 9)), Ok(()));
    assert_eq!(g.number_edges(), 1);
    assert_eq!(targets_and_keys(&g, 0), vec![(0, 1)]);
    assert_eq!(g.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(g.inbound_edges(0).unwrap().collect::<Vec<_>>(), vec![0]);
    assert!(g.contains_edge_keyed(0, 0, 1));
}

// --- Nodes ---------------------------------------------------------------------------------

#[test]
fn test_add_node_issues_the_next_position_in_live() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(3);
    g.remove_node(2).unwrap();
    // Position 2 is a tombstone and is never reissued: the next node sits at 3.
    assert_eq!(g.add_node(30), Ok(3));
    assert_eq!(g.number_nodes(), 3);
    assert!(!g.contains_node(2));
    assert_eq!(g.get_node(3), Some(&30));
    assert_eq!(g.get_last_index(), Some(3));
}

#[test]
fn test_update_node_replaces_the_payload_of_a_live_node() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(2);
    assert_eq!(g.update_node(1, 11), Ok(()));
    assert_eq!(g.get_node(1), Some(&11));
    assert_eq!(g.update_node(2, 22), Err(GraphError::NodeNotFound(2)));
    g.remove_node(0).unwrap();
    assert_eq!(g.update_node(0, 5), Err(GraphError::NodeNotFound(0)));
}

#[test]
fn test_add_root_node_in_live() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(1);
    assert_eq!(g.add_root_node(9), Ok(1));
    assert_eq!(g.get_root_index(), Some(1));
    assert_eq!(g.get_root_node(), Some(&9));
    assert_eq!(g.add_root_node(10), Err(GraphError::RootNodeAlreadyExists));
    g.remove_node(1).unwrap();
    assert_eq!(g.get_root_index(), None);
    assert!(!g.contains_root_node());
}

/// Scenario "A removed node in Live": a node with incoming and outgoing edges is removed; it
/// reads as absent, no traversal yields it, its own traversal is refused, and the edge count
/// excludes its edges. The neighbours' rows still hold its entries, which every read skips.
#[test]
fn test_a_removed_node_in_live_leaves_every_read() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(4);
    // 0 → 1 (two keys), 1 → 2, 3 → 1, 1 → 1, 0 → 2: six edges, four of them touching 1.
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(0, 1, tagged(1, 2)).unwrap();
    g.add_edge(1, 2, tagged(0, 3)).unwrap();
    g.add_edge(3, 1, tagged(0, 4)).unwrap();
    g.add_edge(1, 1, tagged(0, 5)).unwrap();
    g.add_edge(0, 2, tagged(0, 6)).unwrap();
    assert_eq!(g.number_edges(), 6);

    assert_eq!(g.remove_node(1), Ok(()));
    assert!(!g.contains_node(1));
    assert_eq!(g.number_nodes(), 3);
    // Five edges touched 1 (the self-loop counted once): one remains.
    assert_eq!(g.number_edges(), 1);
    assert_eq!(targets_and_keys(&g, 0), vec![(2, 0)]);
    assert_eq!(g.outbound_edges(0).unwrap().collect::<Vec<_>>(), vec![2]);
    assert_eq!(g.outbound_edges(3).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0]);
    assert!(matches!(
        g.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(!g.contains_edge(0, 1));
    assert!(!g.contains_edge_keyed(0, 1, 0));
    assert_eq!(g.get_edges(1), None);
    assert_eq!(
        g.remove_edge(0, 1),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(g.remove_node(1), Err(GraphError::NodeNotFound(1)));
    // The frozen graph drops the stale entries and agrees.
    g.freeze();
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(2, &tagged(0, 6))]);
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0]);
}

/// Row D: two neighbours removed one after the other; the second removal must not subtract
/// the edges to the first again.
#[test]
fn test_two_removals_count_each_edge_once() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(3);
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(1, 2, tagged(0, 2)).unwrap();
    g.add_edge(2, 0, tagged(0, 3)).unwrap();
    g.remove_node(1).unwrap();
    // 0 → 1 and 1 → 2 went with node 1: one edge left, 2 → 0.
    assert_eq!(g.number_edges(), 1);
    g.remove_node(2).unwrap();
    // 2 → 0 goes; the stale 1 → 2 in 2's backward row is not counted again.
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.outbound_edges(0).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(0).unwrap().count(), 0);
}

// --- Edges: rows, capacity and order ---------------------------------------------------------

/// Scenario "A row that outgrows its capacity": node 0 receives edges one at a time past the
/// capacities 1, 2 and 4 (row E: at capacity, one below and one above), every edge is present
/// afterwards, sorted by target then key, and every other node's adjacency is unchanged.
#[test]
fn test_a_row_that_outgrows_its_capacity_keeps_every_edge() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(7);
    g.add_edge(5, 6, tagged(0, 56)).unwrap();
    g.add_edge(6, 5, tagged(0, 65)).unwrap();
    // Targets out of order, so the row's sort is exercised: 4, 2, 6, 1, 3 with kind 0, then
    // 2 again with kind 1: six entries through capacities 1, 2, 4 and 8.
    let order = [(4, 0), (2, 0), (6, 0), (1, 0), (3, 0), (2, 1)];
    let mut expected: Vec<(usize, u8)> = Vec::new();
    for (i, &(t, k)) in order.iter().enumerate() {
        g.add_edge(0, t, tagged(k, i as u32)).unwrap();
        expected.push((t, k));
        expected.sort_unstable();
        assert_eq!(targets_and_keys(&g, 0), expected, "after {} inserts", i + 1);
        assert_eq!(g.number_edges(), 3 + i);
        // The other rows are untouched.
        assert_eq!(targets_and_keys(&g, 5), vec![(6, 0)]);
        assert_eq!(targets_and_keys(&g, 6), vec![(5, 0)]);
        assert_eq!(g.inbound_edges(5).unwrap().collect::<Vec<_>>(), vec![6]);
    }
    // Every target reads node 0 as a source, once per key it carries.
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0, 0]);
    assert_eq!(g.inbound_edges(4).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(g.inbound_edges(6).unwrap().collect::<Vec<_>>(), vec![0, 5]);
    // Payloads are the ones written.
    assert_eq!(
        g.get_edges(0).unwrap(),
        vec![
            (1, &tagged(0, 3)),
            (2, &tagged(0, 1)),
            (2, &tagged(1, 5)),
            (3, &tagged(0, 4)),
            (4, &tagged(0, 0)),
            (6, &tagged(0, 2)),
        ]
    );
}

/// Row D: the last node's row grows at the arena's end, where no row follows it.
#[test]
fn test_the_last_nodes_row_grows() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(4);
    for t in [1, 0, 2] {
        g.add_edge(3, t, tagged(0, t as u32)).unwrap();
    }
    assert_eq!(targets_and_keys(&g, 3), vec![(0, 0), (1, 0), (2, 0)]);
    assert_eq!(g.number_edges(), 3);
    for t in 0..3 {
        assert_eq!(g.inbound_edges(t).unwrap().collect::<Vec<_>>(), vec![3]);
    }
}

/// Row H: a row filled to its capacity exactly, emptied by one and refilled: the spare slot is
/// reused in place, and the order holds.
#[test]
fn test_a_removal_frees_a_slot_the_next_insert_takes() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(4);
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(0, 3, tagged(0, 3)).unwrap();
    // Capacity 2, both slots full.
    assert_eq!(g.remove_edge_keyed(0, 1, 0), Ok(()));
    assert_eq!(targets_and_keys(&g, 0), vec![(3, 0)]);
    assert_eq!(g.add_edge(0, 2, tagged(0, 2)), Ok(()));
    assert_eq!(targets_and_keys(&g, 0), vec![(2, 0), (3, 0)]);
    assert_eq!(g.number_edges(), 2);
    assert_eq!(g.inbound_edges(1).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(2).unwrap().collect::<Vec<_>>(), vec![0]);
}

/// Row C: a put of an existing `(target, key)` replaces the payload in both rows and adds no
/// edge; a put of a new key beside it adds one.
#[test]
fn test_a_repeated_key_replaces_and_a_new_key_adds() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(2);
    g.add_edge(0, 1, tagged(2, 1)).unwrap();
    g.add_edge(0, 1, tagged(2, 7)).unwrap();
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(2, 7))]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    g.add_edge(0, 1, tagged(1, 8)).unwrap();
    assert_eq!(g.number_edges(), 2);
    assert_eq!(targets_and_keys(&g, 0), vec![(1, 1), (1, 2)]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0, 0]);
}

#[test]
fn test_an_edge_to_or_from_an_absent_node_is_refused_in_live() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(2);
    g.remove_node(1).unwrap();
    for (a, b) in [(0, 1), (1, 0), (0, 2), (2, 0)] {
        assert_eq!(
            g.add_edge(a, b, tagged(0, 0)),
            Err(GraphError::EdgeCreationError {
                source: a,
                target: b
            })
        );
    }
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_remove_edge_takes_every_key_of_a_pair_in_live() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(3);
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(0, 1, tagged(1, 2)).unwrap();
    g.add_edge(0, 2, tagged(0, 3)).unwrap();
    g.add_edge(2, 1, tagged(0, 4)).unwrap();
    assert_eq!(g.remove_edge(0, 1), Ok(()));
    assert_eq!(g.number_edges(), 2);
    assert_eq!(targets_and_keys(&g, 0), vec![(2, 0)]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![2]);
    assert_eq!(
        g.remove_edge(0, 1),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(g.remove_edge(3, 1), Err(GraphError::NodeNotFound(3)));
}

#[test]
fn test_remove_edge_keyed_takes_one_key_in_live() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(2);
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(0, 1, tagged(1, 2)).unwrap();
    assert_eq!(g.remove_edge_keyed(0, 1, 0), Ok(()));
    assert_eq!(targets_and_keys(&g, 0), vec![(1, 1)]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    assert_eq!(g.number_edges(), 1);
    assert_eq!(
        g.remove_edge_keyed(0, 1, 0),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(
        g.remove_edge_keyed(2, 1, 0),
        Err(GraphError::NodeNotFound(2))
    );
}

/// Row F: a weight of zero is an edge in the Live state.
#[test]
fn test_a_zero_weight_is_an_edge_in_live() {
    let mut g: UltraGraphWeighted<u8, u64> = live_nodes(2);
    g.add_edge(0, 1, 0).unwrap();
    assert!(g.contains_edge(0, 1));
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &0)]);
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.shortest_weighted_path(0, 1), Ok(Some((vec![0, 1], 0))));
}

/// Scenario "Clearing a Live graph": `clear` leaves an empty Dynamic graph and positions restart
/// at 0.
#[test]
fn test_clear_from_live_leaves_an_empty_dynamic_graph() {
    let mut g: UltraGraphWeighted<u8, Tagged> = live_nodes(3);
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    assert_eq!(g.clear(), Ok(()));
    assert!(!g.is_live());
    assert!(!g.is_frozen());
    assert!(g.is_empty());
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.add_node(1), Ok(0));
}
