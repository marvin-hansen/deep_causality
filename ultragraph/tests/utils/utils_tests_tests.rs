/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The edge-storage accounting of `ultragraph::utils::utils_tests`.
//!
//! Every byte count is derived by hand from the capacities the constructors reserve, on a
//! 64-bit target: a `Vec` header is 24 bytes, `usize` is 8 bytes, and `with_capacity(nodes,
//! Some(k))` reserves the outer forward and backward vectors for `nodes` entries and each
//! node's forward and backward list for `k` entries. The node table is not counted.

use ultragraph::utils::utils_tests::{edge_storage_bytes, memory_per_edge};
use ultragraph::*;

/// A payload whose key is itself, one byte wide.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Rel {
    #[default]
    Parent,
    Child,
}

impl EdgeKey for Rel {
    type Key = Rel;
    fn edge_key(&self) -> Rel {
        *self
    }
}

/// A payload keyed by one of its fields: `kind` identifies the edge, `w` is data.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Tagged {
    kind: u8,
    w: f32,
}

impl EdgeKey for Tagged {
    type Key = u8;
    fn edge_key(&self) -> u8 {
        self.kind
    }
}

/// Three nodes reserved for `per_node` edges each, with the edges 0 → 1 and 1 → 2.
fn three_nodes_two_edges<W: Clone + Default + EdgeKey>(
    per_node: usize,
    w: W,
) -> UltraGraphWeighted<i32, W> {
    let mut g = UltraGraphWeighted::with_capacity(3, Some(per_node));
    for i in 0..3 {
        g.add_node(i).unwrap();
    }
    g.add_edge(0, 1, w.clone()).unwrap();
    g.add_edge(1, 2, w).unwrap();
    g
}

// --- Row A: the empty graph --------------------------------------------------------------

#[test]
fn test_an_empty_graph_holds_no_edge_storage() {
    let g: UltraGraph<i32> = UltraGraph::new();
    // `new` reserves nothing: every vector has capacity 0.
    assert_eq!(edge_storage_bytes(&g), 0);
    assert_eq!(memory_per_edge(&g), None);
}

#[test]
fn test_reserved_nodes_without_edges_count_their_outer_vectors_only() {
    let g: UltraGraph<i32> = UltraGraph::with_capacity(5, Some(3));
    // No node was added, so no per-node list exists; the two outer vectors hold room for five
    // 24-byte list headers each: 2 × 5 × 24 = 240. No edge, so no per-edge figure.
    assert_eq!(edge_storage_bytes(&g), 240);
    assert_eq!(memory_per_edge(&g), None);
}

// --- The specified scenario: three nodes, the edges 0 → 1 and 1 → 2 ------------------------

#[test]
fn test_three_nodes_two_edges_count_every_reserved_list() {
    let g = three_nodes_two_edges(2, ());
    // Outer vectors: 3 headers × 24 bytes, forward and backward: 2 × 72 = 144.
    // Per-node lists: 3 nodes × 2 directions × 2 reserved entries × 8 bytes `(usize, ())`,
    // node 2's forward list and node 0's backward list reserved though empty: 96.
    // Total 240 bytes over 2 edges: 120 bytes per edge.
    assert_eq!(edge_storage_bytes(&g), 240);
    assert_eq!(memory_per_edge(&g), Some(120.0));
}

// --- Rows B and H: one edge filling a list of capacity one exactly -------------------------

#[test]
fn test_one_edge_fills_its_list_exactly() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(2, Some(1));
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    // Outer: 2 × 2 × 24 = 96. Lists: 2 nodes × 2 directions × 1 entry × 8 bytes = 32.
    // 128 bytes over 1 edge.
    assert_eq!(edge_storage_bytes(&g), 128);
    assert_eq!(memory_per_edge(&g), Some(128.0));
}

// --- Capacity, not length ----------------------------------------------------------------

#[test]
fn test_reserved_capacity_counts_where_the_length_does_not() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(3, Some(2));
    for i in 0..3 {
        g.add_node(i).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    // The same 240 bytes as with two edges: every list is reserved for 2 entries whether it
    // holds 0, 1 or 2. (Counting lengths would give 144 + 2 × 8 = 160.) One edge: 240 per edge.
    assert_eq!(edge_storage_bytes(&g), 240);
    assert_eq!(memory_per_edge(&g), Some(240.0));
}

#[test]
fn test_the_outer_vectors_count_their_capacity() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(8, Some(1));
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    // Outer: room for 8 headers each way, 2 × 8 × 24 = 384, though only 2 nodes exist
    // (counting their lengths would give 96). Lists: 2 × 2 × 1 × 8 = 32. 416 over 1 edge.
    assert_eq!(edge_storage_bytes(&g), 416);
    assert_eq!(memory_per_edge(&g), Some(416.0));
}

#[test]
fn test_a_removed_edge_leaves_its_capacity_counted() {
    let mut g = three_nodes_two_edges(2, ());
    g.remove_edge(1, 2).unwrap();
    // Removal shrinks lengths, never capacities: still 240 bytes, now over 1 edge.
    assert_eq!(edge_storage_bytes(&g), 240);
    assert_eq!(memory_per_edge(&g), Some(240.0));
}

// --- Row K: every payload kind -----------------------------------------------------------

#[test]
fn test_a_weight_counts_its_size_forward_and_its_unit_key_backward() {
    let g = three_nodes_two_edges(2, 7_u64);
    // Forward lists hold `(usize, u64)`, 16 bytes: 3 × 2 × 16 = 96, plus the outer 72: 168.
    // Backward lists hold `(usize, ())`, 8 bytes, since `u64` has the unit key: 3 × 2 × 8 = 48,
    // plus 72: 120. Total 288 over 2 edges: 144 per edge.
    assert_eq!(edge_storage_bytes(&g), 288);
    assert_eq!(memory_per_edge(&g), Some(144.0));
}

#[test]
fn test_a_zero_weight_counts_as_any_weight() {
    let g = three_nodes_two_edges(2, 0_u64);
    // A weight of 0 occupies its 8 bytes like any other: the 288 of the weighted fixture.
    assert_eq!(edge_storage_bytes(&g), 288);
}

#[test]
fn test_an_identity_key_counts_its_size_in_both_directions() {
    let g = three_nodes_two_edges(2, Rel::Child);
    // `(usize, Rel)` is 16 bytes (one byte padded to the 8-byte alignment) in both directions,
    // since the key is the payload: 2 × (72 + 3 × 2 × 16) = 2 × 168 = 336 over 2 edges.
    assert_eq!(edge_storage_bytes(&g), 336);
    assert_eq!(memory_per_edge(&g), Some(168.0));
}

#[test]
fn test_a_field_key_counts_the_payload_forward_and_the_key_backward() {
    let g = three_nodes_two_edges(2, Tagged { kind: 3, w: 1.5 });
    // `Tagged` is 8 bytes (u8 padded beside an f32), so `(usize, Tagged)` is 16; the key `u8`
    // gives `(usize, u8)`, also 16 by padding. 2 × (72 + 96) = 336 over 2 edges.
    assert_eq!(edge_storage_bytes(&g), 336);
    assert_eq!(memory_per_edge(&g), Some(168.0));
}

// --- The frozen state --------------------------------------------------------------------

#[test]
fn test_a_frozen_graph_counts_its_csr_arrays() {
    let mut g = three_nodes_two_edges(2, ());
    g.freeze();
    // `freeze` builds, per direction, offsets reserved for slots + 1 = 4 entries, and targets
    // and weights of exactly 2 entries: 4 × 8 + 2 × 8 + 2 × 0 = 48. Both directions: 96 over
    // 2 edges: 48 per edge. The dynamic lists are gone.
    assert_eq!(edge_storage_bytes(&g), 96);
    assert_eq!(memory_per_edge(&g), Some(48.0));
}

#[test]
fn test_a_frozen_weighted_graph_counts_its_weights() {
    let mut g = three_nodes_two_edges(2, 7_u64);
    g.freeze();
    // As above with 2 weights of 8 bytes per direction: 2 × (32 + 16 + 16) = 128 over 2 edges.
    assert_eq!(edge_storage_bytes(&g), 128);
    assert_eq!(memory_per_edge(&g), Some(64.0));
}

#[test]
fn test_an_empty_frozen_graph_holds_its_offset_sentinels_only() {
    let mut g: UltraGraph<i32> = UltraGraph::new();
    g.freeze();
    // An empty CSR holds one offset per direction, `[0]`, in a vector of capacity 1: 2 × 8.
    assert_eq!(edge_storage_bytes(&g), 16);
    assert_eq!(memory_per_edge(&g), None);
}
