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
//! The literals depend on the edge-storage candidate the build selects, so each sits in a
//! constant with its derivation for all three: the per-node `Vec` list, whose header is one
//! `Vec` and whose entries are `(other, value)`; the snapshot-log array (`--features edge-log`),
//! whose header is two `Vec`s and a count, and whose reserved buffer is the log, an entry being
//! `other` beside a put of the value, the removal of a key or the removal of every key; and the
//! indexed list (`--features edge-index`), the `Vec` list beside an optional index, which is
//! absent below 64 entries and counted by the accounting's `BTreeMap` rule above.

use ultragraph::utils::utils_tests::{edge_storage_bytes, memory_per_edge};
use ultragraph::*;

const LOG: bool = cfg!(feature = "edge-log");
const INDEX: bool = cfg!(feature = "edge-index");

/// The list header: a `Vec` (24) for the `Vec` list; two `Vec`s and a `usize` (56) for the
/// snapshot-log array; a `Vec` and an `Option<BTreeMap>` (24 + 32, the option carrying its own
/// tag) for the indexed list, also 56.
const HEADER: usize = if LOG || INDEX { 56 } else { 24 };
/// One reserved entry of a list of `()` values, forward or backward: `(usize, ())` is 8; a log
/// entry of a `()` put or removal is a `usize` and a one-byte tag, padded to 16.
const UNIT: usize = if LOG { 16 } else { 8 };
/// One reserved forward entry of a `u64` weight: `(usize, u64)` is 16; a log entry carrying a
/// `u64` put is a `usize`, a tag and the `u64`, 24.
const WEIGHT: usize = if LOG { 24 } else { 16 };
/// One reserved entry of a one-byte key or identity-keyed payload (`Rel`), forward or backward:
/// `(usize, Rel)` is 16 by padding; a log entry of a one-byte put or removal is also 16.
const BYTE: usize = 16;
/// One reserved forward entry of the 8-byte `Tagged` payload: `(usize, Tagged)` is 16; a log
/// entry of a `Tagged` put is a `usize` beside a 4-aligned tag and the payload, 24.
const TAGGED: usize = if LOG { 24 } else { 16 };

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
    // list headers each: 2 × 5 × HEADER, 240 or 560. No edge, so no per-edge figure.
    assert_eq!(edge_storage_bytes(&g), 2 * 5 * HEADER);
    assert_eq!(memory_per_edge(&g), None);
}

// --- The specified scenario: three nodes, the edges 0 → 1 and 1 → 2 ------------------------

#[test]
fn test_three_nodes_two_edges_count_every_reserved_list() {
    let g = three_nodes_two_edges(2, ());
    // Outer vectors: 3 headers, forward and backward: 2 × 3 × HEADER, 144 or 336.
    // Per-node lists: 3 nodes × 2 directions × 2 reserved `()` entries, node 2's forward list
    // and node 0's backward list reserved though empty: 12 × UNIT, 96 or 192.
    // Total 240 or 528 bytes over 2 edges: 120 or 264 bytes per edge.
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
    // Outer: 2 × 2 × HEADER, 96 or 224. Lists: 2 nodes × 2 directions × 1 `()` entry, 4 × UNIT,
    // 32 or 64. 128 or 288 bytes over 1 edge.
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
    // The same bytes as with two edges, 240 or 528: every list is reserved for 2 entries
    // whether it holds 0, 1 or 2. (Counting lengths would give 144 + 2 × 8 = 160 for the `Vec`
    // list.) One edge, so the whole count per edge.
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
    // Outer: room for 8 headers each way, 2 × 8 × HEADER, 384 or 896, though only 2 nodes exist
    // (counting their lengths would give 96 or 224). Lists: 2 × 2 × 1 `()` entry, 4 × UNIT, 32
    // or 64. 416 or 960 over 1 edge.
    let bytes = 2 * 8 * HEADER + 4 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
}

#[test]
fn test_a_removed_edge_leaves_its_capacity_counted() {
    let mut g = three_nodes_two_edges(2, ());
    g.remove_edge(1, 2).unwrap();
    // Removal shrinks lengths, never capacities: still 240 or 528 bytes, now over 1 edge. In
    // the log, the removal is a second entry in a log reserved for 2.
    let bytes = 2 * 3 * HEADER + 12 * UNIT;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64));
}

// --- Row K: every payload kind -----------------------------------------------------------

#[test]
fn test_a_weight_counts_its_size_forward_and_its_unit_key_backward() {
    let g = three_nodes_two_edges(2, 7_u64);
    // Forward lists hold `u64` entries, 3 × 2 × WEIGHT, 96 or 144, plus the outer 3 × HEADER.
    // Backward lists hold `()` entries, since `u64` has the unit key: 3 × 2 × UNIT, 48 or 96,
    // plus 3 × HEADER. Total 288 or 576 over 2 edges: 144 or 288 per edge.
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
    // 2 × (3 × HEADER + 3 × 2 × BYTE), 336 or 528, over 2 edges.
    let bytes = 2 * 3 * HEADER + 12 * BYTE;
    assert_eq!(edge_storage_bytes(&g), bytes);
    assert_eq!(memory_per_edge(&g), Some(bytes as f64 / 2.0));
}

#[test]
fn test_a_field_key_counts_the_payload_forward_and_the_key_backward() {
    let g = three_nodes_two_edges(2, Tagged { kind: 3, w: 1.5 });
    // `Tagged` is 8 bytes (u8 padded beside an f32): forward entries 3 × 2 × TAGGED, 96 or 144;
    // the key `u8` gives one-byte backward entries, 3 × 2 × BYTE = 96. With the outer vectors:
    // 336 or 576 over 2 edges.
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

// --- The reserved buffers follow the candidate's growth (row E of the storage candidates) ----

/// Node 0 of eighteen nodes receives one `()` edge to each of nodes 1 to `k`, for `k` one below,
/// at and one above the snapshot-log trigger of 16, the graph built without an edge hint so
/// every list grows by pushes. The byte count pins where each candidate's buffers grow.
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

#[test]
fn test_the_buffers_grow_where_the_candidate_grows_them() {
    // A `Vec` pushed into grows 4, 8, 16, 32 entries. Outer vectors: 2 × 18 × HEADER.
    // `Vec` list: node 0's forward list holds 15, 16 or 17 entries at capacity 16, 16 or 32,
    // times 8; each target's backward list holds one entry at capacity 4 × 8 = 32.
    //   864 + 128 + 15 × 32 = 1,472; 864 + 128 + 16 × 32 = 1,504; 864 + 256 + 17 × 32 = 1,664.
    // Snapshot-log array: node 0's log holds 15 puts at capacity 16 × 16 = 256; the 16th put
    // reaches the trigger and compacts: a snapshot of exactly 16 entries × 8 = 128 beside the
    // cleared log, which keeps its capacity, 384; the 17th put lands in that log, still 384.
    // Each target's backward log holds one put at capacity 4 × 16 = 64.
    //   2,016 + 256 + 15 × 64 = 3,232; 2,016 + 384 + 16 × 64 = 3,424; 2,016 + 384 + 17 × 64 = 3,488.
    // Indexed list: the `Vec` list's buffers, no index below 64 entries, behind 56-byte headers:
    //   2,016 + 608; 2,016 + 640; 2,016 + 800.
    let outer = 2 * 18 * HEADER;
    let expected = if LOG {
        [outer + 1_216, outer + 1_408, outer + 1_472]
    } else {
        [outer + 608, outer + 640, outer + 800]
    };
    assert_eq!(
        bytes_after_k_edges(15),
        expected[0],
        "one below the trigger"
    );
    assert_eq!(bytes_after_k_edges(16), expected[1], "at the trigger");
    assert_eq!(
        bytes_after_k_edges(17),
        expected[2],
        "one above the trigger"
    );
}

/// Node 0 receives 15 edges and then a put that replaces the payload of its first edge: the
/// replacement is the 16th write, so the snapshot-log array compacts, and the snapshot it builds
/// holds the 15 current edges, not 16. The `Vec` list replaces in place and grows nothing.
#[test]
fn test_a_replacement_does_not_grow_the_snapshot() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=15 {
        g.add_edge(0, t, ()).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    assert_eq!(g.number_edges(), 15);
    // `Vec` list: as after 15 edges, 1,472, and the indexed list likewise behind its headers.
    // Snapshot-log array: the forward log of node 0 holds 16 entries at capacity 16 × 16 = 256
    // and compacts into a snapshot of exactly 15 × 8 = 120, 376 with the cleared log; the
    // replacement touches no backward list, 15 × 64 = 960; 2,016 + 376 + 960 = 3,352. A
    // snapshot sized for 16 would read 3,360.
    let expected = 2 * 18 * HEADER + if LOG { 1_336 } else { 608 };
    assert_eq!(edge_storage_bytes(&g), expected);
}

/// Node 0 receives 15 edges and then a removal of a pair that has no edge: nothing is written,
/// so the buffers read as after the 15 edges. A storage that logged the failed removal would
/// have reached the trigger and compacted.
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
    // As after 15 edges: 1,472 for the `Vec` list, 2,016 + 608 indexed; 3,232 for the
    // snapshot-log array, whose forward log of node 0 still holds 15 puts short of the trigger.
    let expected = 2 * 18 * HEADER + if LOG { 1_216 } else { 608 };
    assert_eq!(edge_storage_bytes(&g), expected);
}

/// Node 0 receives 15 edges and then a keyed removal of its first as the 16th write: the
/// snapshot-log array compacts into a snapshot of the 14 current edges; the `Vec` list shifts
/// its entries and keeps its capacity.
#[test]
fn test_a_keyed_removal_shrinks_the_snapshot_by_one() {
    let mut g: UltraGraph<i32> = UltraGraph::with_capacity(18, None);
    for i in 0..18 {
        g.add_node(i).unwrap();
    }
    for t in 1..=15 {
        g.add_edge(0, t, ()).unwrap();
    }
    g.remove_edge_keyed(0, 1, ()).unwrap();
    assert_eq!(g.number_edges(), 14);
    // `Vec` list: 1,472, as after 15 edges, and the indexed list likewise behind its headers.
    // Snapshot-log array: node 0's forward log holds 15 puts and the removal, 16 entries at
    // capacity 16 × 16 = 256, and compacts into a snapshot of exactly 14 × 8 = 112: 368; node
    // 1's backward log holds its put and the removal at capacity 4 × 16 = 64, as every other
    // target's: 15 × 64 = 960; 2,016 + 368 + 960 = 3,344. A count left at 15 would read 3,352,
    // one raised to 16 would read 3,360.
    let expected = 2 * 18 * HEADER + if LOG { 1_328 } else { 608 };
    assert_eq!(edge_storage_bytes(&g), expected);
}

/// Node 0 carries two keys to node 1 and one to each of nodes 2 to 14, 15 edges of an
/// identity-keyed payload; the removal of the pair `(0, 1)` is the 16th write and takes two
/// edges, so the snapshot-log array compacts into a snapshot of the 13 current edges.
#[test]
fn test_a_pair_removal_shrinks_the_snapshot_by_every_key() {
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
    // Entries are 16 bytes in both directions for either candidate (BYTE). Outer vectors:
    // 2 × 15 × HEADER, 720 or 1,680.
    // `Vec` list: node 0's forward list grew to capacity 16 × 16 = 256 and keeps it after the
    // removal; node 1's backward list held two entries at capacity 4 × 16 = 64, as each of the
    // thirteen other targets' lists: 14 × 64 = 896. 720 + 256 + 896 = 1,872.
    // Snapshot-log array: node 0's forward log holds 15 puts and the removal of every key, 16
    // entries at capacity 16 × 16 = 256, and compacts into a snapshot of exactly 13 × 16 = 208:
    // 464; node 1's backward log holds two puts and the removal at capacity 4 × 16 = 64, as the
    // thirteen others': 896. 1,680 + 464 + 896 = 3,040. A count lowered by one key only would
    // read 3,056; one raised by the two keys 3,104; one divided by them 2,944.
    // Indexed list: the `Vec` list's buffers behind 56-byte headers, 1,680 + 1,152.
    let expected = 2 * 15 * HEADER + if LOG { 1_360 } else { 1_152 };
    assert_eq!(edge_storage_bytes(&g), expected);
}

// --- The index follows the degree threshold (the indexed-lists candidate, row E) -------------

/// Node 0 of 65 nodes, built without an edge hint, receives one `()` edge to each of nodes 1 to
/// 64; the byte count is read at 63 and 64 edges, then after keyed removals down to 32 and 31.
/// The indexed list's index appears at 64 and leaves below 32; the other candidates' buffers
/// only grow.
#[test]
fn test_the_index_is_counted_from_the_threshold_until_below_half_of_it() {
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

    // Outer vectors: 2 × 65 × HEADER: 3,120 for the `Vec` list, 7,280 for the other two.
    // `Vec` list: node 0's forward list, pushed 63 and 64 times, sits at capacity 64 × 8 = 512
    // and keeps it through the removals; each target's backward list holds its one entry at
    // capacity 4 × 8 = 32 and keeps that capacity once emptied: 63 × 32, then 64 × 32 = 2,048.
    //   3,120 + 512 + 2,016 = 5,648; then 5,680 at 64, 32 and 31 edges.
    // Snapshot-log array: node 0's log compacts at the 16th, 32nd and 64th put (the trigger
    // is the snapshot's length once that exceeds 16): at 63 puts a snapshot of 32 × 8 = 256
    // beside a log of 31 at capacity 32 × 16 = 512, 768; at 64 a snapshot of 64 × 8 = 512 beside
    // the cleared log at capacity 32, 1,024; 32 removals fill that log exactly, 1,024; the 33rd
    // grows it to 64 × 16, 1,536. Each target's backward log holds a put, then the removal, at
    // capacity 4 × 16 = 64: 63 × 64, then 64 × 64 = 4,096.
    //   7,280 + 768 + 4,032 = 12,080; 7,280 + 1,024 + 4,096 = 12,400; 12,400; 7,280 + 1,536 + 4,096 = 12,912.
    // Indexed list: the `Vec` list's buffers, plus from 64 entries the index: an outer map of
    // one entry per target, each a `usize` key, a 24-byte inner map and an 8-byte child pointer,
    // 40 bytes, with a 16-byte node header per 11 entries; and one inner map per target holding
    // one entry of a `()` key, a `usize` position and a child pointer, 16 bytes, plus one header.
    //   64 targets: 64 × 40 + 6 × 16 = 2,656 and 64 × 32 = 2,048: 4,704.
    //   32 targets: 32 × 40 + 3 × 16 = 1,328 and 32 × 32 = 1,024: 2,352. Below 32: none.
    //   7,280 + 512 + 2,016 = 9,808; 7,280 + 512 + 4,704 + 2,048 = 14,544;
    //   7,280 + 512 + 2,352 + 2,048 = 12,192; 7,280 + 512 + 2,048 = 9,840.
    let expected = if LOG {
        [12_080, 12_400, 12_400, 12_912]
    } else if INDEX {
        [9_808, 14_544, 12_192, 9_840]
    } else {
        [5_648, 5_680, 5_680, 5_680]
    };
    assert_eq!([at_63, at_64, at_32, at_31], expected);
}
