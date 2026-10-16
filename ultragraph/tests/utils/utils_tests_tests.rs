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
//!
//! Each size the counts rest on sits in a named constant with its derivation: the per-node
//! list's header is one `Vec`, and its entries are `(other, value)` pairs.

use ultragraph::utils::utils_tests::{edge_storage_bytes, memory_per_edge};
use ultragraph::*;

/// The list header: one `Vec`, 24 bytes.
const HEADER: usize = 24;
/// One entry of a list of `()` values, forward or backward: `(usize, ())`, 8 bytes.
const UNIT: usize = 8;
/// One forward entry of a `u64` weight: `(usize, u64)`, 16 bytes.
const WEIGHT: usize = 16;
/// One entry of a one-byte key or identity-keyed payload (`Rel`), forward or backward:
/// `(usize, Rel)`, 16 bytes by padding.
const BYTE: usize = 16;
/// One forward entry of the 8-byte `Tagged` payload: `(usize, Tagged)`, 16 bytes.
const TAGGED: usize = 16;

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
    // list headers each: 2 × 5 × 24 = 240. No edge, so no per-edge figure.
    assert_eq!(edge_storage_bytes(&g), 2 * 5 * HEADER);
    assert_eq!(memory_per_edge(&g), None);
}

// --- The specified scenario: three nodes, the edges 0 → 1 and 1 → 2 ------------------------

#[test]
fn test_three_nodes_two_edges_count_every_reserved_list() {
    let g = three_nodes_two_edges(2, ());
    // Outer vectors: 3 headers, forward and backward: 2 × 3 × 24 = 144.
    // Per-node lists: 3 nodes × 2 directions × 2 reserved `()` entries, node 2's forward list
    // and node 0's backward list reserved though empty: 12 × 8 = 96.
    // Total 240 bytes over 2 edges: 120 bytes per edge.
    let bytes = 2 * 3 * HEADER + 12 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64 / 2.0));
}

// --- Rows B and H: one edge filling a list of capacity one exactly -------------------------

#[test]
fn test_one_edge_fills_its_list_exactly() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(2, Some(1));
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    // Outer: 2 × 2 × 24 = 96. Lists: 2 nodes × 2 directions × 1 `()` entry, 4 × 8 = 32.
    // 128 bytes over 1 edge.
    let bytes = 2 * 2 * HEADER + 4 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
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
    // holds 0, 1 or 2. (Counting lengths would give 144 + 2 × 8 = 160.) One edge, so the whole
    // count per edge.
    let bytes = 2 * 3 * HEADER + 12 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
}

#[test]
fn test_the_outer_vectors_count_their_capacity() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(8, Some(1));
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, ()).unwrap();
    // Outer: room for 8 headers each way, 2 × 8 × 24 = 384, though only 2 nodes exist (counting
    // their lengths would give 96). Lists: 2 × 2 × 1 `()` entry, 4 × 8 = 32. 416 over 1 edge.
    let bytes = 2 * 8 * HEADER + 4 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
}

#[test]
fn test_a_removed_edge_leaves_its_capacity_counted() {
    let mut g = three_nodes_two_edges(2, ());
    g.remove_edge(1, 2).unwrap();
    // Removal shrinks lengths, never capacities: still 240 bytes, now over 1 edge.
    let bytes = 2 * 3 * HEADER + 12 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
}

// --- Row K: every payload kind -----------------------------------------------------------

#[test]
fn test_a_weight_counts_its_size_forward_and_its_unit_key_backward() {
    let g = three_nodes_two_edges(2, 7_u64);
    // Forward lists hold `u64` entries, 3 × 2 × 16 = 96, plus the outer 72. Backward lists
    // hold `()` entries, since `u64` has the unit key: 3 × 2 × 8 = 48, plus 72. Total 288 over
    // 2 edges: 144 per edge.
    let bytes = 2 * 3 * HEADER + 6 * WEIGHT + 6 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64 / 2.0));
}

#[test]
fn test_a_zero_weight_counts_as_any_weight() {
    let g = three_nodes_two_edges(2, 0_u64);
    // A weight of 0 occupies its 8 bytes like any other: the bytes of the weighted fixture.
    assert_eq!(
        edge_storage_bytes(&g),
        2 * 3 * HEADER + 6 * WEIGHT + 6 * UNIT
    );
}

#[test]
fn test_an_identity_key_counts_its_size_in_both_directions() {
    let g = three_nodes_two_edges(2, Rel::Child);
    // A one-byte entry is 16 bytes in both directions, since the key is the payload:
    // 2 × (72 + 3 × 2 × 16) = 336 over 2 edges.
    let bytes = 2 * 3 * HEADER + 12 * BYTE;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64 / 2.0));
}

#[test]
fn test_a_field_key_counts_the_payload_forward_and_the_key_backward() {
    let g = three_nodes_two_edges(2, Tagged { kind: 3, w: 1.5 });
    // `Tagged` is 8 bytes (u8 padded beside an f32): forward entries 3 × 2 × 16 = 96; the key
    // `u8` gives one-byte backward entries, 3 × 2 × 16 = 96. With the outer vectors: 336 over
    // 2 edges.
    let bytes = 2 * 3 * HEADER + 6 * TAGGED + 6 * BYTE;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64 / 2.0));
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

// --- The lists grow as they are pushed, and keep their capacity (row E) --------------------

/// Node 0 of eighteen nodes receives one `()` edge to each of nodes 1 to `k`, the graph built
/// without an edge hint so every list grows by pushes.
fn bytes_after_k_edges(k: usize) -> usize {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=k {
        g.add_edge(0, t, ()).unwrap();
    }
    assert_eq!(g.number_edges(), k);
    edge_storage_bytes(&g)
}

/// Eighteen nodes, the outer vectors reserved for them: 2 × 18 × 24 = 864 bytes.
const EIGHTEEN_OUTER: usize = 864;

#[test]
fn test_the_lists_grow_as_they_are_pushed() {
    // A `Vec` pushed into grows 4, 8, 16, 32 entries. Node 0's forward list holds 15, 16 or 17
    // entries at capacity 16, 16 or 32, times 8; each target's backward list holds one entry at
    // capacity 4 × 8 = 32.
    //   864 + 128 + 15 × 32 = 1,472; 864 + 128 + 16 × 32 = 1,504; 864 + 256 + 17 × 32 = 1,664.
    assert_eq!(bytes_after_k_edges(15), EIGHTEEN_OUTER + 608);
    assert_eq!(bytes_after_k_edges(16), EIGHTEEN_OUTER + 640);
    assert_eq!(bytes_after_k_edges(17), EIGHTEEN_OUTER + 800);
}

/// Node 0 receives 15 edges and then a put that replaces the payload of its first edge: the
/// replacement writes in place and grows nothing.
#[test]
fn test_a_replacement_grows_nothing() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=15 {
        g.add_edge(0, t, ()).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    assert_eq!(g.number_edges(), 15);
    // As after 15 edges: 1,472.
    assert_eq!(edge_storage_bytes(&g), EIGHTEEN_OUTER + 608);
}

/// Node 0 receives 15 edges and then a removal of a pair that has no edge: nothing is written,
/// so the buffers read as after the 15 edges.
#[test]
fn test_a_failed_pair_removal_writes_nothing() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=15 {
        g.add_edge(0, t, ()).unwrap();
    }
    assert!(g.remove_edge(0, 16).is_err());
    assert_eq!(edge_storage_bytes(&g), EIGHTEEN_OUTER + 608);
}

/// Node 0 receives 15 edges and then a keyed removal of its first: the list shifts its entries
/// and keeps its capacity, and the target's emptied backward list keeps its capacity too.
#[test]
fn test_a_keyed_removal_keeps_the_capacity() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=15 {
        g.add_edge(0, t, ()).unwrap();
    }
    g.remove_edge_keyed(0, 1, ()).unwrap();
    assert_eq!(g.number_edges(), 14);
    // As after 15 edges: 1,472.
    assert_eq!(edge_storage_bytes(&g), EIGHTEEN_OUTER + 608);
}

/// Node 0 carries two keys to node 1 and one to each of nodes 2 to 14, 15 edges of an
/// identity-keyed payload; the removal of the pair `(0, 1)` takes two edges and keeps every
/// capacity.
#[test]
fn test_a_pair_removal_keeps_the_capacity() {
    let mut g: UltraGraphWeighted<i32, Rel> = UltraGraphWeighted::with_capacity(15, None);
    for i in 0..15 {
        g.add_node(i).unwrap();
    }
    g.add_edge(0, 1, Rel::Parent).unwrap();
    g.add_edge(0, 1, Rel::Child).unwrap();
    for t in 2..=14 {
        g.add_edge(0, t, Rel::Parent).unwrap();
    }
    g.remove_edge(0, 1).unwrap();
    assert_eq!(g.number_edges(), 13);
    // Entries are 16 bytes in both directions. Outer vectors: 2 × 15 × 24 = 720. Node 0's
    // forward list grew to capacity 16 × 16 = 256 and keeps it after the removal; node 1's
    // backward list held two entries at capacity 4 × 16 = 64, as each of the thirteen other
    // targets' lists: 14 × 64 = 896. 720 + 256 + 896 = 1,872.
    assert_eq!(edge_storage_bytes(&g), 1_872);
}

/// Node 0 of 65 nodes receives one `()` edge to each of nodes 1 to 64; the byte count is read
/// at 63 and 64 edges, then after keyed removals down to 32 and 31: the lists grow at 64 and
/// keep their capacity through the removals.
#[test]
fn test_capacity_stays_through_growth_and_removals() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(65, None);
    for i in 0..65 {
        g.add_node(i).unwrap();
    }
    for t in 1..=63 {
        g.add_edge(0, t, ()).unwrap();
    }
    let at_63 = edge_storage_bytes(&g);
    g.add_edge(0, 64, ()).unwrap();
    let at_64 = edge_storage_bytes(&g);
    for t in 1..=32 {
        g.remove_edge_keyed(0, t, ()).unwrap();
    }
    assert_eq!(g.number_edges(), 32);
    let at_32 = edge_storage_bytes(&g);
    g.remove_edge_keyed(0, 33, ()).unwrap();
    let at_31 = edge_storage_bytes(&g);
    // Outer vectors: 2 × 65 × 24 = 3,120. Node 0's forward list, pushed 63 and 64 times, sits
    // at capacity 64 × 8 = 512 and keeps it through the removals; each target's backward list
    // holds its one entry at capacity 4 × 8 = 32 and keeps that capacity once emptied: 63 × 32,
    // then 64 × 32 = 2,048.
    //   3,120 + 512 + 2,016 = 5,648; then 5,680 at 64, 32 and 31 edges.
    assert_eq!([at_63, at_64, at_32, at_31], [5_648, 5_680, 5_680, 5_680]);
}
