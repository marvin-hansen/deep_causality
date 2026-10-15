/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EdgeKey, GraphMut, GraphView, UltraGraph, UltraGraphContainer};

// Helper to create a graph for testing.
// 0 -> 1 -> 2
// ^----|    |-> 3
// |         |
// +---------+
pub fn get_cyclic_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap(); // Cycle
    g.add_edge(2, 3, ()).unwrap();
    g.freeze();
    g
}

// Helper to create a DAG.
// 0 -> 1 -> 2 -> 3
pub fn get_acyclic_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();
    g.freeze();
    g
}

/// The bytes the graph's edge storage holds in its current state: the capacity of every buffer
/// the storage owns, each times its element size, both directions included and the node table
/// excluded. Deterministic: two graphs built by the same calls report the same count.
pub fn edge_storage_bytes<N, W>(graph: &UltraGraphContainer<N, W>) -> usize
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    graph.edge_storage_bytes()
}

/// `edge_storage_bytes` divided by the number of edges, or `None` for a graph with no edge.
pub fn memory_per_edge<N, W>(graph: &UltraGraphContainer<N, W>) -> Option<f64>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    let edges = graph.number_edges();
    (edges > 0).then(|| edge_storage_bytes(graph) as f64 / edges as f64)
}
