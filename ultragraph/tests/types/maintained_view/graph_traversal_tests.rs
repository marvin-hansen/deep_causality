/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Traversal on a maintained view: one entry per passing edge, in either mode.

use ultragraph::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    #[default]
    A,
    B,
}

impl EdgeKey for Kind {
    type Key = Kind;
    fn edge_key(&self) -> Kind {
        *self
    }
}

type Small = (fn(&u8) -> bool, fn(&Kind) -> bool);

fn small() -> Small {
    (|n| *n < 10, |_| true)
}

/// Nodes 0..4 with payloads 1, 2, 30, 4 (position 2 fails); edges 0 → 1 (both kinds),
/// 1 → 2, 2 → 3, 0 → 3, 3 → 0.
fn graph() -> UltraGraphContainer<u8, Kind, Small> {
    let mut g = UltraGraphContainer::new();
    for p in [1, 2, 30, 4] {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 1, Kind::B).unwrap();
    g.add_edge(0, 1, Kind::A).unwrap();
    g.add_edge(1, 2, Kind::A).unwrap();
    g.add_edge(2, 3, Kind::A).unwrap();
    g.add_edge(0, 3, Kind::A).unwrap();
    g.add_edge(3, 0, Kind::A).unwrap();
    g
}

#[test]
fn test_outbound_edges_list_one_entry_per_passing_edge_sorted() {
    let mut g = graph();
    for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
        let id = g.maintain(small(), mode);
        let view = g.view(id).unwrap();
        assert_eq!(
            view.outbound_edges(0).unwrap().collect::<Vec<_>>(),
            vec![1, 1, 3]
        );
        assert_eq!(
            view.outbound_edges(1).unwrap().count(),
            0,
            "1 → 2 fails at 2"
        );
        assert_eq!(view.outbound_edges(3).unwrap().collect::<Vec<_>>(), vec![0]);
    }
}

#[test]
fn test_inbound_edges_list_one_entry_per_passing_edge_sorted() {
    let mut g = graph();
    let id = g.maintain(small(), ParallelEdges::Merge);
    let view = g.view(id).unwrap();
    assert_eq!(
        view.inbound_edges(1).unwrap().collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(
        view.inbound_edges(3).unwrap().collect::<Vec<_>>(),
        vec![0],
        "2 → 3 fails at 2"
    );
    assert_eq!(view.inbound_edges(0).unwrap().collect::<Vec<_>>(), vec![3]);
}

#[test]
fn test_traversal_of_a_failing_or_absent_node_is_refused() {
    let mut g = graph();
    let id = g.maintain(small(), ParallelEdges::Merge);
    let view = g.view(id).unwrap();
    assert!(matches!(
        view.outbound_edges(2),
        Err(GraphError::NodeNotFound(2))
    ));
    assert!(matches!(
        view.inbound_edges(2),
        Err(GraphError::NodeNotFound(2))
    ));
    assert!(matches!(
        view.outbound_edges(7),
        Err(GraphError::NodeNotFound(7))
    ));
}

/// A traversal follows the mutations in the Live state too.
#[test]
fn test_traversal_follows_mutations_in_live() {
    let mut g = graph();
    let id = g.maintain(small(), ParallelEdges::Merge);
    g.make_live();
    g.update_node(2, 3).unwrap();
    assert_eq!(
        g.view(id)
            .unwrap()
            .outbound_edges(1)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(
        g.view(id)
            .unwrap()
            .inbound_edges(3)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    g.remove_edge_keyed(0, 1, Kind::B).unwrap();
    assert_eq!(
        g.view(id)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    g.remove_node(3).unwrap();
    assert_eq!(
        g.view(id)
            .unwrap()
            .outbound_edges(0)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(g.view(id).unwrap().inbound_edges(0).unwrap().count(), 0);
}
