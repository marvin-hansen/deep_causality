/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Live → Frozen (spec: ultragraph-three-states, "Every transition keeps every position, node
//! and edge"), through the container.

use ultragraph::*;

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

/// Every read of a graph, collected so two states can be compared; lists are sorted, since the
/// dynamic state's order is not specified.
fn snapshot(g: &UltraGraphWeighted<u8, Tagged>) -> Vec<String> {
    let bound = g.get_last_index().map_or(0, |last| last + 2);
    let mut out = vec![
        format!("nodes {}", g.number_nodes()),
        format!("edges {}", g.number_edges()),
        format!("root {:?}", g.get_root_index()),
        format!("last {:?}", g.get_last_index()),
    ];
    for p in 0..bound {
        out.push(format!("node {p} {:?}", g.get_node(p)));
        let edges = g.get_edges(p).map(|mut e| {
            e.sort_by_key(|(t, w)| (*t, w.kind));
            e
        });
        out.push(format!("out {p} {edges:?}"));
        let fwd = g.outbound_edges(p).map(|i| sorted(i.collect()));
        out.push(format!("fwd {p} {fwd:?}"));
        let bwd = g.inbound_edges(p).map(|i| sorted(i.collect()));
        out.push(format!("bwd {p} {bwd:?}"));
    }
    out
}

fn sorted(mut v: Vec<usize>) -> Vec<usize> {
    v.sort_unstable();
    v
}

/// A graph with a tombstone, parallel keyed edges and a root, built dynamic.
fn build() -> UltraGraphWeighted<u8, Tagged> {
    let mut g = UltraGraphWeighted::new();
    assert_eq!(g.add_root_node(0).unwrap(), 0);
    for p in 1..6u8 {
        g.add_node(p).unwrap();
    }
    g.add_edge(0, 1, tagged(0, 1)).unwrap();
    g.add_edge(0, 1, tagged(2, 2)).unwrap();
    g.add_edge(0, 1, tagged(1, 3)).unwrap();
    g.add_edge(1, 2, tagged(0, 4)).unwrap();
    g.add_edge(2, 0, tagged(0, 5)).unwrap();
    g.add_edge(4, 5, tagged(0, 6)).unwrap();
    g.add_edge(5, 5, tagged(1, 7)).unwrap();
    g.remove_node(3).unwrap();
    g
}

/// Freezing a Live graph that was adopted and not changed reads as the frozen graph it came
/// from.
#[test]
fn test_freeze_after_make_live_reads_as_before() {
    let mut g = build();
    g.freeze();
    let frozen = snapshot(&g);
    g.make_live();
    assert!(g.is_live());
    assert_eq!(snapshot(&g), frozen, "live reads as frozen");
    g.freeze();
    assert!(g.is_frozen());
    assert_eq!(snapshot(&g), frozen, "frozen again reads as before");
}

/// Freezing after mutations in Live: rows that moved, rows with spare capacity, a removed node
/// whose entries still sit in its neighbours' rows, all close into a CSR that reads as a graph
/// built dynamic with the same operations and frozen.
#[test]
fn test_freeze_after_live_mutations_matches_the_dynamic_build() {
    let mut live = build();
    live.make_live();
    let mut reference = build();
    for g in [&mut live, &mut reference] {
        g.add_edge(2, 4, tagged(0, 8)).unwrap();
        g.add_edge(2, 1, tagged(0, 9)).unwrap();
        g.add_edge(2, 5, tagged(1, 10)).unwrap();
        g.remove_edge_keyed(0, 1, 2).unwrap();
        g.remove_node(5).unwrap();
        g.add_node(60).unwrap();
        g.add_edge(6, 0, tagged(0, 11)).unwrap();
        g.update_node(4, 44).unwrap();
    }
    live.freeze();
    reference.freeze();
    assert_eq!(snapshot(&live), snapshot(&reference));
    // Seven built, three added, one key removed, three gone with node 5, one added: seven.
    assert_eq!(live.number_edges(), 7);
    assert_eq!(
        live.strongly_connected_components(),
        reference.strongly_connected_components()
    );
}

#[test]
fn test_freeze_of_an_empty_live_graph() {
    let mut g: UltraGraphWeighted<u8, Tagged> = UltraGraphWeighted::new();
    g.make_live();
    g.freeze();
    assert!(g.is_frozen());
    assert!(g.is_empty());
    assert_eq!(g.number_edges(), 0);
}

/// Scenario "A round through every state": Dynamic → Live → Frozen → Live → Dynamic, every
/// read equal after each transition.
#[test]
fn test_a_round_through_every_state_keeps_every_read() {
    let mut g = build();
    let before = snapshot(&g);
    g.make_live();
    assert!(g.is_live());
    assert_eq!(snapshot(&g), before, "Dynamic → Live");
    g.freeze();
    assert!(g.is_frozen());
    assert_eq!(snapshot(&g), before, "Live → Frozen");
    g.make_live();
    assert!(g.is_live());
    assert_eq!(snapshot(&g), before, "Frozen → Live");
    g.unfreeze();
    assert!(!g.is_live() && !g.is_frozen());
    assert_eq!(snapshot(&g), before, "Live → Dynamic");
}

#[test]
fn test_make_live_is_idempotent_and_freeze_from_frozen_is_a_no_op() {
    let mut g = build();
    g.make_live();
    let live = snapshot(&g);
    g.make_live();
    assert!(g.is_live());
    assert_eq!(snapshot(&g), live);
    g.freeze();
    g.freeze();
    assert!(g.is_frozen());
    assert_eq!(snapshot(&g), live);
}
