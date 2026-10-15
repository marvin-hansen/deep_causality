/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The contract every edge-storage candidate of `ultragraph-live-state-maintained-views` meets,
//! exercised through the graph's public API: the edges of a node are a map from `(target, key)`
//! to the payload, and every operation reads and writes that map. The independent source is a
//! `BTreeMap` model kept beside the graph and compared after every step.
//!
//! The snapshot-log candidate compacts a node's log once it holds `max(snapshot.len(), MIN_LOG)`
//! entries, `MIN_LOG` being 16; the tests below drive a node to one entry below, at and above
//! that trigger, with a snapshot shorter and longer than `MIN_LOG`, through `freeze` and
//! `unfreeze`, which rebuild every list as a snapshot with an empty log. The `Vec` lists have
//! no trigger and pass the same tests.

use deep_causality_rand::{Rng, Xoshiro256};
use std::collections::BTreeMap;
use std::fmt::Debug;
use ultragraph::*;

/// The snapshot-log candidate's `MIN_LOG`.
const TRIGGER: usize = 16;

// --- Payload kinds (row K) -----------------------------------------------------------------

/// An identity key with three values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Rel {
    #[default]
    Parent,
    Child,
    Peer,
}

impl EdgeKey for Rel {
    type Key = Rel;
    fn edge_key(&self) -> Rel {
        *self
    }
}

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

// --- The model ---------------------------------------------------------------------------

/// The edges as the contract defines them: `(source, target, key)` to payload, over a table of
/// live positions.
struct Model<W: EdgeKey> {
    live: Vec<bool>,
    edges: BTreeMap<(usize, usize, W::Key), W>,
}

impl<W: EdgeKey + Clone + PartialEq + Debug> Model<W> {
    fn new(n: usize) -> Self {
        Self {
            live: vec![true; n],
            edges: BTreeMap::new(),
        }
    }

    fn add_edge(&mut self, a: usize, b: usize, w: W) {
        self.edges.insert((a, b, w.edge_key()), w);
    }

    /// The number of edges from `a` to `b` removed.
    fn remove_edge(&mut self, a: usize, b: usize) -> usize {
        let before = self.edges.len();
        self.edges.retain(|&(s, t, _), _| !(s == a && t == b));
        before - self.edges.len()
    }

    fn remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) -> bool {
        self.edges.remove(&(a, b, key)).is_some()
    }

    fn remove_node(&mut self, p: usize) {
        self.live[p] = false;
        self.edges.retain(|&(s, t, _), _| s != p && t != p);
    }

    fn edges_of(&self, a: usize) -> Vec<(usize, W)> {
        self.edges
            .iter()
            .filter(|((s, _, _), _)| *s == a)
            .map(|((_, t, _), w)| (*t, w.clone()))
            .collect()
    }

    fn sources_of(&self, b: usize) -> Vec<usize> {
        let mut v: Vec<usize> = self
            .edges
            .keys()
            .filter(|(_, t, _)| *t == b)
            .map(|(s, _, _)| *s)
            .collect();
        v.sort_unstable();
        v
    }
}

/// Every read of the graph against the model: per live node its edges, keyed and unkeyed
/// membership for every pair and key in `keys`, both traversals, and the edge count.
fn check<W: EdgeKey + Clone + Default + PartialEq + Debug>(
    g: &UltraGraphWeighted<u8, W>,
    m: &Model<W>,
    keys: &[W::Key],
    step: usize,
) where
    W::Key: Debug,
{
    assert_eq!(g.number_edges(), m.edges.len(), "step {step}: number_edges");
    for a in 0..m.live.len() {
        assert_eq!(g.contains_node(a), m.live[a], "step {step}: node {a}");
        if !m.live[a] {
            assert!(
                g.get_edges(a).is_none(),
                "step {step}: edges of removed {a}"
            );
            assert!(
                g.outbound_edges(a).is_err(),
                "step {step}: outbound of removed {a}"
            );
            assert!(
                g.inbound_edges(a).is_err(),
                "step {step}: inbound of removed {a}"
            );
            continue;
        }
        let mut listed: Vec<(usize, W)> = g
            .get_edges(a)
            .unwrap()
            .into_iter()
            .map(|(t, w)| (t, w.clone()))
            .collect();
        listed.sort_by_key(|(t, w)| (*t, w.edge_key()));
        assert_eq!(listed, m.edges_of(a), "step {step}: get_edges({a})");

        let mut out: Vec<usize> = g.outbound_edges(a).unwrap().collect();
        out.sort_unstable();
        let expected_out: Vec<usize> = m.edges_of(a).into_iter().map(|(t, _)| t).collect();
        assert_eq!(out, expected_out, "step {step}: outbound_edges({a})");

        let mut inbound: Vec<usize> = g.inbound_edges(a).unwrap().collect();
        inbound.sort_unstable();
        assert_eq!(inbound, m.sources_of(a), "step {step}: inbound_edges({a})");

        for b in 0..m.live.len() {
            let any = keys.iter().any(|k| m.edges.contains_key(&(a, b, *k)));
            assert_eq!(
                g.contains_edge(a, b),
                any,
                "step {step}: contains_edge({a}, {b})"
            );
            for &k in keys {
                assert_eq!(
                    g.contains_edge_keyed(a, b, k),
                    m.edges.contains_key(&(a, b, k)),
                    "step {step}: contains_edge_keyed({a}, {b}, {k:?})"
                );
            }
        }
    }
}

/// `n` nodes at positions `0..n`, payload equal to the position.
fn nodes<W: Clone + Default + EdgeKey>(n: usize) -> UltraGraphWeighted<u8, W> {
    let mut g = UltraGraphWeighted::new();
    for p in 0..n {
        assert_eq!(g.add_node(p as u8).unwrap(), p);
    }
    g
}

/// Random operations over `n` nodes for `steps` steps, checked against the model after each:
/// keyed adds (45 %), keyed removals (15 %), pair removals (10 %), a node removal followed by a
/// fresh node at a new position (5 %), and a `freeze`/`unfreeze` round trip (5 %), which turns
/// every node's log into a snapshot; the rest are repeated reads. A node's log crosses the
/// trigger many times over 1,500 steps on five nodes.
fn drive<W, F>(seed: u64, n: usize, steps: usize, keys: &[W::Key], mut draw: F)
where
    W: Clone + Default + EdgeKey + PartialEq + Debug,
    W::Key: Debug,
    F: FnMut(&mut Xoshiro256) -> W,
{
    let mut rng = Xoshiro256::from_seed(seed);
    let mut g: UltraGraphWeighted<u8, W> = nodes(n);
    let mut m: Model<W> = Model::new(n);
    let mut frozen_round_trips = 0;

    for step in 0..steps {
        let roll = rng.random_range(0..100usize);
        let bound = m.live.len();
        let a = rng.random_range(0..bound);
        let b = rng.random_range(0..bound);
        match roll {
            0..=44 => {
                let w = draw(&mut rng);
                if m.live[a] && m.live[b] {
                    m.add_edge(a, b, w.clone());
                    assert_eq!(
                        g.add_edge(a, b, w),
                        Ok(()),
                        "step {step}: add_edge({a}, {b})"
                    );
                } else {
                    assert_eq!(
                        g.add_edge(a, b, w),
                        Err(GraphError::EdgeCreationError {
                            source: a,
                            target: b
                        })
                    );
                }
            }
            45..=59 => {
                let k = keys[rng.random_range(0..keys.len())];
                let expected = if !m.live[a] {
                    Err(GraphError::NodeNotFound(a))
                } else if m.remove_edge_keyed(a, b, k) {
                    Ok(())
                } else {
                    Err(GraphError::EdgeNotFoundError {
                        source: a,
                        target: b,
                    })
                };
                assert_eq!(
                    g.remove_edge_keyed(a, b, k),
                    expected,
                    "step {step}: remove_edge_keyed({a}, {b}, {k:?})"
                );
            }
            60..=69 => {
                let expected = if !m.live[a] {
                    Err(GraphError::NodeNotFound(a))
                } else if m.remove_edge(a, b) > 0 {
                    Ok(())
                } else {
                    Err(GraphError::EdgeNotFoundError {
                        source: a,
                        target: b,
                    })
                };
                assert_eq!(
                    g.remove_edge(a, b),
                    expected,
                    "step {step}: remove_edge({a}, {b})"
                );
            }
            70..=74 => {
                if m.live[a] {
                    m.remove_node(a);
                    assert_eq!(g.remove_node(a), Ok(()), "step {step}: remove_node({a})");
                    let p = m.live.len();
                    m.live.push(true);
                    assert_eq!(g.add_node(p as u8), Ok(p), "step {step}: add_node");
                } else {
                    assert_eq!(g.remove_node(a), Err(GraphError::NodeNotFound(a)));
                }
            }
            75..=79 => {
                g.freeze();
                g.unfreeze();
                frozen_round_trips += 1;
            }
            _ => {}
        }
        check(&g, &m, keys, step);
    }
    assert!(
        frozen_round_trips > 0,
        "the driver never rebuilt the lists as snapshots"
    );
}

#[test]
fn test_model_based_keyed_edges_with_unit_payloads() {
    drive::<(), _>(0x5EED_2001, 5, 1_500, &[()], |_| ());
}

#[test]
fn test_model_based_keyed_edges_with_weights() {
    drive::<u64, _>(0x5EED_2002, 5, 1_500, &[()], |rng| {
        rng.random_range(0..1_000u64)
    });
}

#[test]
fn test_model_based_keyed_edges_with_identity_keys() {
    let kinds = [Rel::Parent, Rel::Child, Rel::Peer];
    drive::<Rel, _>(0x5EED_2003, 5, 1_500, &kinds, |rng| {
        kinds[rng.random_range(0..3usize)]
    });
}

#[test]
fn test_model_based_keyed_edges_with_field_keys() {
    drive::<Tagged, _>(0x5EED_2004, 5, 1_500, &[0, 1, 2], |rng| {
        tagged(rng.random_range(0..3u8), rng.random_range(0..100u32))
    });
}

// --- One node across the compaction trigger (rows C, E, H) --------------------------------

/// A graph of `1 + snapshot + extra` nodes whose node 0 carries one edge to each of the nodes
/// `1..=snapshot`, rebuilt as a snapshot by a `freeze`/`unfreeze` round trip; the nodes above
/// `snapshot` are free targets for the operations that follow.
fn node_with_snapshot(snapshot: usize, extra: usize) -> UltraGraphWeighted<u8, Tagged> {
    let mut g = nodes(1 + snapshot + extra);
    for t in 1..=snapshot {
        g.add_edge(0, t, tagged(0, t as u32)).unwrap();
    }
    g.freeze();
    g.unfreeze();
    assert_eq!(g.number_edges(), snapshot);
    g
}

/// `ops` keyed inserts out of node 0 to the free targets, kinds cycling through 1 and 2, after
/// which the node holds its snapshot and every inserted edge.
fn insert_then_check(snapshot: usize, ops: usize) {
    let g_targets = ops.div_ceil(2);
    let mut g = node_with_snapshot(snapshot, g_targets);
    let mut expected: BTreeMap<(usize, u8), u32> =
        (1..=snapshot).map(|t| ((t, 0), t as u32)).collect();
    for i in 0..ops {
        let t = 1 + snapshot + i / 2;
        let kind = 1 + (i % 2) as u8;
        g.add_edge(0, t, tagged(kind, 100 + i as u32)).unwrap();
        expected.insert((t, kind), 100 + i as u32);
    }
    let mut listed: Vec<((usize, u8), u32)> = g
        .get_edges(0)
        .unwrap()
        .into_iter()
        .map(|(t, w)| ((t, w.kind), w.w))
        .collect();
    listed.sort_unstable();
    let expected: Vec<((usize, u8), u32)> = expected.into_iter().collect();
    assert_eq!(listed, expected, "snapshot {snapshot}, {ops} inserts");
    assert_eq!(g.number_edges(), snapshot + ops);
    // Every target reads back from its own side as well: one inbound entry per key it carries.
    for ((t, kind), _) in &expected {
        assert!(g.contains_edge_keyed(0, *t, *kind));
        let keys_of_t = expected.iter().filter(|((u, _), _)| u == t).count();
        assert_eq!(
            g.inbound_edges(*t).unwrap().collect::<Vec<_>>(),
            vec![0; keys_of_t]
        );
    }
}

/// Row E with a snapshot shorter than `MIN_LOG`: the trigger is `MIN_LOG`, so 15, 16 and 17
/// operations sit one below, at and one above it.
#[test]
fn test_inserts_around_the_trigger_with_a_short_snapshot() {
    for ops in [TRIGGER - 1, TRIGGER, TRIGGER + 1] {
        insert_then_check(5, ops);
    }
}

/// Row C: a snapshot exactly `MIN_LOG` long, where the two arms of the trigger coincide.
#[test]
fn test_inserts_around_the_trigger_with_a_snapshot_of_the_trigger_length() {
    for ops in [TRIGGER - 1, TRIGGER, TRIGGER + 1] {
        insert_then_check(TRIGGER, ops);
    }
}

/// Row E with a snapshot longer than `MIN_LOG`: the trigger is the snapshot's length, 40, so
/// 39, 40 and 41 operations sit one below, at and one above it.
#[test]
fn test_inserts_around_the_trigger_with_a_long_snapshot() {
    for ops in [39, 40, 41] {
        insert_then_check(40, ops);
    }
}

/// Row H: a second crossing. Operations beyond the trigger keep going for twice the trigger
/// and more, and every edge is present at the end.
#[test]
fn test_inserts_across_two_triggers() {
    insert_then_check(5, 2 * TRIGGER + 3);
    insert_then_check(TRIGGER, 3 * TRIGGER);
}

// --- Operations whose order the log must keep ----------------------------------------------

/// A put, a removal and a put of the same keyed edge, within one log: the newest decides, so
/// the edge exists with the last payload.
#[test]
fn test_a_put_after_a_removal_of_the_same_key_is_kept() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(2);
    g.add_edge(0, 1, tagged(1, 10)).unwrap();
    g.remove_edge_keyed(0, 1, 1).unwrap();
    g.add_edge(0, 1, tagged(1, 20)).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(1, 20))]);
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
}

/// The same sequence driven across the trigger, so the put, the removal and the newer put
/// are resolved by a compaction rather than read from the log.
#[test]
fn test_a_put_after_a_removal_survives_a_compaction() {
    let mut g = node_with_snapshot(3, 1);
    let free = 4;
    g.add_edge(0, free, tagged(1, 10)).unwrap();
    g.remove_edge_keyed(0, free, 1).unwrap();
    g.add_edge(0, free, tagged(1, 20)).unwrap();
    // 13 further replacements of an existing payload carry the log past the trigger of 16.
    for i in 0..TRIGGER - 3 {
        g.add_edge(0, 1, tagged(0, 1_000 + i as u32)).unwrap();
    }
    assert!(g.contains_edge_keyed(0, free, 1));
    let edges: BTreeMap<usize, Tagged> = g
        .get_edges(0)
        .unwrap()
        .into_iter()
        .map(|(t, w)| (t, *w))
        .collect();
    assert_eq!(edges[&free], tagged(1, 20));
    assert_eq!(edges[&1], tagged(0, 1_000 + (TRIGGER - 4) as u32));
    assert_eq!(g.number_edges(), 4);
}

/// A removal of every key of a pair followed by a put of one key: the pair holds that key alone.
#[test]
fn test_a_pair_removal_followed_by_a_put_leaves_the_new_key_alone() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(2);
    g.add_edge(0, 1, tagged(1, 1)).unwrap();
    g.add_edge(0, 1, tagged(2, 2)).unwrap();
    g.remove_edge(0, 1).unwrap();
    g.add_edge(0, 1, tagged(2, 3)).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(2, 3))]);
    assert!(!g.contains_edge_keyed(0, 1, 1));
    assert_eq!(g.number_edges(), 1);
}

/// Repeated puts of one keyed edge within one log, carried past the trigger: the compaction
/// resolves them as one edge with the newest payload, never as duplicates.
#[test]
fn test_repeated_puts_of_one_edge_compact_to_one_edge() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(2);
    for i in 0..2 * TRIGGER + 1 {
        g.add_edge(0, 1, tagged(7, i as u32)).unwrap();
    }
    assert_eq!(
        g.get_edges(0).unwrap(),
        vec![(1, &tagged(7, 2 * TRIGGER as u32))]
    );
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    g.freeze();
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap().len(), 1);
}

/// Row F: a weight of zero is a payload, not a removal: it is stored, read back and counted,
/// before and after a compaction.
#[test]
fn test_a_zero_weight_is_an_edge() {
    let mut g: UltraGraphWeighted<u8, u64> = nodes(2);
    g.add_edge(0, 1, 0).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &0)]);
    assert!(g.contains_edge(0, 1));
    assert_eq!(g.number_edges(), 1);
    for _ in 0..TRIGGER + 1 {
        g.add_edge(0, 1, 0).unwrap();
    }
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &0)]);
    assert_eq!(g.number_edges(), 1);
    g.add_edge(0, 0, 0).unwrap();
    assert_eq!(g.number_edges(), 2);
    assert!(g.contains_edge(0, 0));
}

/// A node removed while a neighbour's snapshot still names it: the neighbour lists it no more,
/// in either direction, and the frozen graph agrees.
#[test]
fn test_a_removed_node_leaves_the_snapshots_of_its_neighbours() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(3);
    g.add_edge(0, 1, tagged(1, 1)).unwrap();
    g.add_edge(0, 1, tagged(2, 2)).unwrap();
    g.add_edge(1, 2, tagged(1, 3)).unwrap();
    g.add_edge(2, 1, tagged(1, 4)).unwrap();
    g.freeze();
    g.unfreeze();
    g.remove_node(1).unwrap();
    assert_eq!(g.number_edges(), 0);
    assert!(g.get_edges(0).unwrap().is_empty());
    assert!(g.get_edges(2).unwrap().is_empty());
    assert_eq!(g.outbound_edges(0).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(2).unwrap().count(), 0);
    assert!(!g.contains_edge(0, 1));
    g.freeze();
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.outbound_edges(0).unwrap().count(), 0);
}

// --- Rows A and B ---------------------------------------------------------------------------

/// A node with no edge: nothing is listed and every removal is reported.
#[test]
fn test_a_node_without_edges_answers_empty() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(2);
    assert!(g.get_edges(0).unwrap().is_empty());
    assert_eq!(g.outbound_edges(0).unwrap().count(), 0);
    assert_eq!(g.inbound_edges(1).unwrap().count(), 0);
    assert!(!g.contains_edge(0, 1));
    assert!(!g.contains_edge_keyed(0, 1, 0));
    assert_eq!(
        g.remove_edge(0, 1),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(
        g.remove_edge_keyed(0, 1, 0),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(g.number_edges(), 0);
}

/// One edge through every operation: present after its put, replaced by a put of the same key,
/// kept beside a second key, gone after its keyed removal, gone with the other after a pair
/// removal.
#[test]
fn test_one_edge_through_every_operation() {
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(2);
    g.add_edge(0, 1, tagged(1, 1)).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(1, 1))]);
    g.add_edge(0, 1, tagged(1, 2)).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(1, 2))]);
    assert_eq!(g.number_edges(), 1);
    g.add_edge(0, 1, tagged(2, 3)).unwrap();
    assert_eq!(g.number_edges(), 2);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0, 0]);
    g.remove_edge_keyed(0, 1, 1).unwrap();
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &tagged(2, 3))]);
    assert_eq!(g.inbound_edges(1).unwrap().collect::<Vec<_>>(), vec![0]);
    g.add_edge(0, 1, tagged(1, 4)).unwrap();
    g.remove_edge(0, 1).unwrap();
    assert!(g.get_edges(0).unwrap().is_empty());
    assert_eq!(g.inbound_edges(1).unwrap().count(), 0);
    assert_eq!(g.number_edges(), 0);
}

/// The parts round trip after a log has been resolved: `to_parts` lists the current edges of
/// every node, and `from_parts` rebuilds the same graph.
#[test]
fn test_parts_list_the_current_edges_after_a_log() {
    let mut d: DynamicGraph<u8, Tagged> = DynamicGraph::new();
    d.add_node(0).unwrap();
    d.add_node(1).unwrap();
    for i in 0..TRIGGER + 2 {
        d.add_edge(0, 1, tagged((i % 2) as u8, i as u32)).unwrap();
    }
    d.remove_edge_keyed(0, 1, 0).unwrap();
    let (nodes, edges, root) = d.to_parts();
    assert_eq!(edges[0], vec![(1, tagged(1, (TRIGGER + 1) as u32))]);
    assert!(edges[1].is_empty());
    let rebuilt = DynamicGraph::from_parts((nodes, edges, root));
    assert_eq!(rebuilt.number_edges(), 1);
    assert_eq!(
        rebuilt.get_edges(0).unwrap(),
        vec![(1, &tagged(1, (TRIGGER + 1) as u32))]
    );
    assert_eq!(
        rebuilt.inbound_edges(1).unwrap().collect::<Vec<_>>(),
        vec![0]
    );
}

/// `from_parts` given lists in no particular order: every keyed lookup, keyed removal and pair
/// removal on them answers from the entries, whatever order they came in.
#[test]
fn test_parts_given_out_of_order_are_looked_up_correctly() {
    let nodes = vec![Some(0u8), Some(1), Some(2), Some(3)];
    let edges = vec![
        vec![
            (3, tagged(1, 1)),
            (1, tagged(2, 2)),
            (2, tagged(0, 3)),
            (1, tagged(0, 4)),
        ],
        vec![(0, tagged(2, 5))],
        vec![],
        vec![(2, tagged(1, 6)), (0, tagged(1, 7))],
    ];
    let mut d: DynamicGraph<u8, Tagged> = DynamicGraph::from_parts((nodes, edges, None));
    assert_eq!(d.number_edges(), 7);
    for (a, b, k) in [
        (0, 3, 1),
        (0, 1, 2),
        (0, 2, 0),
        (0, 1, 0),
        (1, 0, 2),
        (3, 2, 1),
        (3, 0, 1),
    ] {
        assert!(d.contains_edge_keyed(a, b, k), "({a}, {b}, {k})");
    }
    assert!(!d.contains_edge_keyed(0, 3, 0));
    assert!(!d.contains_edge_keyed(0, 2, 1));
    assert_eq!(d.remove_edge_keyed(0, 1, 2), Ok(()));
    assert!(!d.contains_edge_keyed(0, 1, 2));
    assert!(d.contains_edge_keyed(0, 1, 0));
    assert_eq!(d.remove_edge(3, 0), Ok(()));
    assert!(!d.contains_edge(3, 0));
    assert_eq!(d.number_edges(), 5);
    let mut inbound: Vec<usize> = d.inbound_edges(0).unwrap().collect();
    inbound.sort_unstable();
    assert_eq!(inbound, vec![1]);
}

// --- A hub across the index threshold (the indexed-lists candidate, rows C, E, H) -----------

/// The indexed-lists candidate's `HUB_DEGREE`: a list builds its index at this many entries
/// and drops it below half of it.
const HUB_DEGREE: usize = 64;

/// Random operations on one hub, node 0, against `targets` targets and the kinds 0, 1 and 2,
/// checked against the model after each, in three phases: a growing phase (puts 60 %, keyed
/// removals 20 %, pair removals 6 %), a shrinking phase (20 %, 55 %, 10 %) and a growing phase
/// again, each with a `freeze`/`unfreeze` round trip and `compact_ids` with no tombstone at
/// 5 % apiece, both of which rebuild the hub's list from entries; the rest repeated reads. The
/// hub's degree crosses `HUB_DEGREE` upward in each growing phase and falls below half of it in
/// the shrinking one; the model is the `BTreeMap` of `drive`.
fn drive_hub(seed: u64, targets: usize, steps: usize) {
    let mut rng = Xoshiro256::from_seed(seed);
    let n = targets + 1;
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(n);
    let mut m: Model<Tagged> = Model::new(n);
    let keys = [0u8, 1, 2];
    let mut peak = 0;
    let mut trough = usize::MAX;
    let mut crossed_up = 0;
    let mut below_half = 0;

    for step in 0..steps {
        let roll = rng.random_range(0..100usize);
        let t = 1 + rng.random_range(0..targets);
        let k = keys[rng.random_range(0..3usize)];
        let before = m.edges_of(0).len();
        let shrinking = (2 * steps / 5..7 * steps / 10).contains(&step);
        let (puts, keyed, pairs) = if shrinking {
            (20, 75, 85)
        } else {
            (60, 80, 86)
        };
        let op = if roll < puts {
            0
        } else if roll < keyed {
            1
        } else if roll < pairs {
            2
        } else if roll < pairs + 5 {
            3
        } else if roll < pairs + 10 {
            4
        } else {
            5
        };
        match op {
            0 => {
                let w = tagged(k, rng.random_range(0..1_000u32));
                m.add_edge(0, t, w);
                assert_eq!(g.add_edge(0, t, w), Ok(()), "step {step}: add_edge(0, {t})");
            }
            1 => {
                let expected = if m.remove_edge_keyed(0, t, k) {
                    Ok(())
                } else {
                    Err(GraphError::EdgeNotFoundError {
                        source: 0,
                        target: t,
                    })
                };
                assert_eq!(g.remove_edge_keyed(0, t, k), expected, "step {step}");
            }
            2 => {
                let expected = if m.remove_edge(0, t) > 0 {
                    Ok(())
                } else {
                    Err(GraphError::EdgeNotFoundError {
                        source: 0,
                        target: t,
                    })
                };
                assert_eq!(g.remove_edge(0, t), expected, "step {step}");
            }
            3 => {
                g.freeze();
                g.unfreeze();
            }
            4 => {
                let table = g.compact_ids().unwrap();
                assert!(table.iter().enumerate().all(|(p, new)| *new == Some(p)));
            }
            _ => {}
        }
        let after = m.edges_of(0).len();
        if before < HUB_DEGREE && after >= HUB_DEGREE {
            crossed_up += 1;
        }
        if before >= HUB_DEGREE / 2 && after < HUB_DEGREE / 2 {
            below_half += 1;
        }
        peak = peak.max(after);
        trough = trough.min(after);
        check(&g, &m, &keys, step);
    }
    assert!(
        peak > HUB_DEGREE + 8,
        "the hub never rose clearly above the threshold: {peak}"
    );
    assert!(
        trough < HUB_DEGREE / 2 - 8,
        "the hub never fell clearly below half: {trough}"
    );
    assert!(
        crossed_up >= 2 && below_half >= 1,
        "crossings: {crossed_up} up, {below_half} below half"
    );
}

#[test]
fn test_model_based_hub_across_the_index_threshold() {
    drive_hub(0x5EED_2005, 40, 2_500);
}

/// The hub grows one keyed edge at a time to one below, at and one above `HUB_DEGREE`, with
/// every keyed read and the pair test answered at each size; then it shrinks by keyed removals
/// to half, one below half and further, the same reads answered; then it grows across the
/// threshold again. Targets carry one to three keys, so entries and targets differ.
#[test]
fn test_a_hub_across_the_threshold_in_both_directions() {
    let targets = HUB_DEGREE + 8;
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(targets + 1);
    let mut m: Model<Tagged> = Model::new(targets + 1);
    let keys = [0u8, 1, 2];
    // Edge i goes to target 1 + i / 2 with kind i % 2, so every second target carries two keys.
    let edge = |i: usize| (1 + i / 2, (i % 2) as u8);
    let mut count = 0;
    let grow_to = |g: &mut UltraGraphWeighted<u8, Tagged>,
                   m: &mut Model<Tagged>,
                   count: &mut usize,
                   size: usize| {
        while *count < size {
            let (t, k) = edge(*count);
            let w = tagged(k, *count as u32);
            m.add_edge(0, t, w);
            g.add_edge(0, t, w).unwrap();
            *count += 1;
        }
        check(g, m, &keys, size);
    };
    grow_to(&mut g, &mut m, &mut count, HUB_DEGREE - 1);
    grow_to(&mut g, &mut m, &mut count, HUB_DEGREE);
    grow_to(&mut g, &mut m, &mut count, HUB_DEGREE + 1);
    grow_to(&mut g, &mut m, &mut count, HUB_DEGREE + 16);

    // A pair removal on the indexed hub takes both keys of its target.
    assert_eq!(g.remove_edge(0, 1), Ok(()));
    assert_eq!(m.remove_edge(0, 1), 2);
    check(&g, &m, &keys, 1_000);

    // Keyed removals, newest first, down to half, one below half and to a handful.
    let shrink_to = |g: &mut UltraGraphWeighted<u8, Tagged>, m: &mut Model<Tagged>, size: usize| {
        while m.edges_of(0).len() > size {
            let (_, t, k) = *m.edges.keys().next_back().unwrap();
            assert!(m.remove_edge_keyed(0, t, k));
            g.remove_edge_keyed(0, t, k).unwrap();
        }
        check(g, m, &keys, size);
    };
    shrink_to(&mut g, &mut m, HUB_DEGREE / 2 + 1);
    shrink_to(&mut g, &mut m, HUB_DEGREE / 2);
    shrink_to(&mut g, &mut m, HUB_DEGREE / 2 - 1);
    shrink_to(&mut g, &mut m, 3);

    // Back across the threshold, with the removed pair re-added along the way.
    let mut count = m.edges_of(0).len();
    for i in 0..HUB_DEGREE + 5 {
        let (t, k) = (1 + (i % targets), (i % 3) as u8);
        let w = tagged(k, 5_000 + i as u32);
        m.add_edge(0, t, w);
        g.add_edge(0, t, w).unwrap();
        count += 1;
        if count == HUB_DEGREE - 1 || count == HUB_DEGREE || count == HUB_DEGREE + 1 {
            check(&g, &m, &keys, 2_000 + count);
        }
    }
    check(&g, &m, &keys, 3_000);
}

/// A hub above the threshold loses a target node and is compacted: every keyed lookup on the
/// hub answers at the remapped positions, and keyed removal and insertion still work.
#[test]
fn test_an_indexed_hub_survives_a_removal_and_a_compaction() {
    let targets = HUB_DEGREE + 6;
    let mut g: UltraGraphWeighted<u8, Tagged> = nodes(targets + 1);
    for t in 1..=targets {
        g.add_edge(0, t, tagged(1, t as u32)).unwrap();
    }
    g.add_edge(0, 5, tagged(2, 500)).unwrap();
    assert_eq!(g.number_edges(), targets + 1);
    // Removing target 3 leaves a tombstone below every other target.
    g.remove_node(3).unwrap();
    let table = g.compact_ids().unwrap();
    assert_eq!(table[3], None);
    for (t, new) in table.iter().enumerate().skip(1) {
        if t == 3 {
            assert_eq!(*new, None);
            continue;
        }
        let new = new.unwrap();
        assert!(g.contains_edge_keyed(0, new, 1), "target {t} at {new}");
        assert_eq!(g.get_node(new), Some(&(t as u8)));
    }
    assert!(g.contains_edge_keyed(0, table[5].unwrap(), 2));
    assert_eq!(g.number_edges(), targets);
    assert_eq!(g.remove_edge_keyed(0, table[5].unwrap(), 2), Ok(()));
    assert!(!g.contains_edge_keyed(0, table[5].unwrap(), 2));
    assert!(g.contains_edge_keyed(0, table[5].unwrap(), 1));
    assert_eq!(g.add_edge(0, table[7].unwrap(), tagged(0, 7)), Ok(()));
    assert!(g.contains_edge_keyed(0, table[7].unwrap(), 0));
    assert_eq!(g.number_edges(), targets);
    // The frozen graph agrees.
    g.freeze();
    assert_eq!(g.number_edges(), targets);
    assert_eq!(g.outbound_edges(0).unwrap().count(), targets);
}

/// `from_parts` given a hub above the threshold in no particular order: keyed lookups, a keyed
/// removal and a pair removal answer from the entries, and `unfreeze` of such a hub likewise.
#[test]
fn test_parts_and_unfreeze_rebuild_a_hub_above_the_threshold() {
    let targets = HUB_DEGREE + 10;
    let nodes: Vec<Option<u8>> = (0..=targets).map(|p| Some(p as u8)).collect();
    let mut hub: Vec<(usize, Tagged)> = (1..=targets).map(|t| (t, tagged(0, t as u32))).collect();
    hub.extend(
        (1..=targets)
            .step_by(3)
            .map(|t| (t, tagged(1, 100 + t as u32))),
    );
    hub.reverse();
    let total = hub.len();
    let mut edges = vec![hub];
    edges.extend((1..=targets).map(|_| Vec::new()));
    let mut d: DynamicGraph<u8, Tagged> = DynamicGraph::from_parts((nodes, edges, None));
    assert_eq!(d.number_edges(), total);
    for t in 1..=targets {
        assert!(d.contains_edge_keyed(0, t, 0), "target {t}");
        assert_eq!(
            d.contains_edge_keyed(0, t, 1),
            (t - 1) % 3 == 0,
            "target {t} key 1"
        );
    }
    assert_eq!(d.remove_edge_keyed(0, 4, 1), Ok(()));
    assert!(!d.contains_edge_keyed(0, 4, 1));
    assert!(d.contains_edge_keyed(0, 4, 0));
    assert_eq!(d.remove_edge(0, 7), Ok(()));
    assert!(!d.contains_edge(0, 7));
    assert_eq!(d.number_edges(), total - 3);

    let frozen = d.freeze();
    let mut back = frozen.unfreeze();
    assert_eq!(back.number_edges(), total - 3);
    assert!(back.contains_edge_keyed(0, 10, 1));
    assert_eq!(back.remove_edge_keyed(0, 10, 1), Ok(()));
    assert!(!back.contains_edge_keyed(0, 10, 1));
    assert_eq!(back.add_edge(0, 7, tagged(2, 7)), Ok(()));
    assert!(back.contains_edge_keyed(0, 7, 2));
    assert_eq!(back.number_edges(), total - 3);
}
