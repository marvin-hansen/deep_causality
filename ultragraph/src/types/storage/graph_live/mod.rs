/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::CsrAdjacency;
use crate::{CsmGraph, EdgeKey};

pub(crate) mod adjacency;
mod graph_algo;
mod graph_algo_centrality;
mod graph_algo_pathfinder;
mod graph_algo_structural;
mod graph_algo_topological;
mod graph_freeze;
mod graph_mut;
mod graph_traversal;
mod graph_unfreeze;
mod graph_view;
mod memory;

/// One node's row in an arena: its entries occupy `start..start + len` of the arena, with room
/// up to `start + cap`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Row {
    pub(crate) start: usize,
    pub(crate) len: usize,
    pub(crate) cap: usize,
}

/// One direction's edges: the other ends and the payloads side by side, a row per position, each
/// row sorted by other end then key. A row that outgrows its capacity is rebuilt at the arena's
/// end with twice the capacity, at least one, and leaves a hole, which `freeze` closes.
#[derive(Clone, Debug)]
pub(crate) struct Arena<W> {
    pub(crate) targets: Vec<usize>,
    pub(crate) weights: Vec<W>,
    pub(crate) rows: Vec<Row>,
}

impl<W: Clone + Default + EdgeKey> Arena<W> {
    /// Adopts a CSR: one row per position over its arrays, each at its exact length.
    pub(crate) fn adopt(csr: CsrAdjacency<W>) -> Self {
        let rows = csr
            .offsets
            .windows(2)
            .map(|pair| Row {
                start: pair[0],
                len: pair[1] - pair[0],
                cap: pair[1] - pair[0],
            })
            .collect();
        Self {
            targets: csr.targets,
            weights: csr.weights,
            rows,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            targets: Vec::new(),
            weights: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// A row for a new position, with no entry and no capacity.
    pub(crate) fn push_row(&mut self) {
        self.rows.push(Row {
            start: self.targets.len(),
            len: 0,
            cap: 0,
        });
    }

    /// The live part of row `v`: its other ends and its payloads.
    pub(crate) fn row(&self, v: usize) -> (&[usize], &[W]) {
        let row = self.rows[v];
        (
            &self.targets[row.start..row.start + row.len],
            &self.weights[row.start..row.start + row.len],
        )
    }

    /// The run of row `v`'s entries with `other` as their other end, as offsets within the row.
    pub(crate) fn run(&self, v: usize, other: usize) -> (usize, usize) {
        let (targets, _) = self.row(v);
        let lo = targets.partition_point(|t| *t < other);
        let hi = lo + targets[lo..].partition_point(|t| *t <= other);
        (lo, hi)
    }

    /// The offset of `(other, key)` within row `v`, or the offset it would be inserted at.
    pub(crate) fn search(&self, v: usize, other: usize, key: W::Key) -> Result<usize, usize> {
        let (lo, hi) = self.run(v, other);
        let (_, weights) = self.row(v);
        let keys = &weights[lo..hi];
        match keys.iter().position(|w| w.edge_key() == key) {
            Some(i) => Ok(lo + i),
            None => Err(lo + keys.partition_point(|w| w.edge_key() < key)),
        }
    }

    /// Inserts `(other, weight)` at offset `at` of row `v`, shifting the entries after it; a
    /// full row is rebuilt at the arena's end with twice its capacity, at least one.
    pub(crate) fn insert_at(&mut self, v: usize, at: usize, other: usize, weight: W) {
        let row = self.rows[v];
        if row.len < row.cap {
            let from = row.start + at;
            let end = row.start + row.len;
            self.targets.copy_within(from..end, from + 1);
            for i in (from..end).rev() {
                self.weights[i + 1] = self.weights[i].clone();
            }
            self.targets[from] = other;
            self.weights[from] = weight;
            self.rows[v].len += 1;
            return;
        }
        let cap = (2 * row.cap).max(1);
        let start = self.targets.len();
        for i in 0..row.len + 1 {
            let (t, w) = if i < at {
                (
                    self.targets[row.start + i],
                    self.weights[row.start + i].clone(),
                )
            } else if i == at {
                (other, weight.clone())
            } else {
                (
                    self.targets[row.start + i - 1],
                    self.weights[row.start + i - 1].clone(),
                )
            };
            self.targets.push(t);
            self.weights.push(w);
        }
        for _ in row.len + 1..cap {
            self.targets.push(0);
            self.weights.push(W::default());
        }
        self.rows[v] = Row {
            start,
            len: row.len + 1,
            cap,
        };
    }

    /// Removes the entries at offsets `from..to` of row `v`, shifting the rest left; the
    /// capacity stays.
    pub(crate) fn remove_run(&mut self, v: usize, from: usize, to: usize) {
        let row = self.rows[v];
        let (a, b, end) = (row.start + from, row.start + to, row.start + row.len);
        self.targets.copy_within(b..end, a);
        for i in b..end {
            self.weights[a + i - b] = self.weights[i].clone();
        }
        self.rows[v].len -= to - from;
    }

    /// Empties row `v`; its capacity stays as a hole.
    pub(crate) fn clear_row(&mut self, v: usize) {
        self.rows[v].len = 0;
    }
}

/// The Live state: mutable row by row and analysable in place. Positions are permanent, as in
/// the other states; a removed node leaves a tombstone, its entries in its neighbours' rows are
/// skipped by every read until the rows are rebuilt, and the edge count is kept exact.
#[derive(Clone, Debug)]
pub struct LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    // Node payloads by position; a removed node leaves `None` and its position is never issued
    // again. The entry doubles as the mask the readers consult.
    nodes: Vec<Option<N>>,
    live_nodes: usize,
    forward: Arena<W>,
    backward: Arena<W>,
    // The number of edges between live nodes; a node removal subtracts its live degree.
    live_edges: usize,
    root_index: Option<usize>,
}

impl<N, W> LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Adopts a frozen graph's CSR arrays as the arenas, one row per position with no spare
    /// capacity, O(V).
    pub(crate) fn from_frozen(frozen: CsmGraph<N, W>) -> Self {
        let (nodes, forward, backward, root_index, live_nodes) = frozen.into_parts();
        let live_edges = forward.targets.len();
        Self {
            nodes,
            live_nodes,
            forward: Arena::adopt(forward),
            backward: Arena::adopt(backward),
            live_edges,
            root_index,
        }
    }

    pub fn root_index(&self) -> Option<usize> {
        self.root_index
    }

    /// The number of positions in the node table, tombstones included.
    pub(crate) fn node_slots(&self) -> usize {
        self.nodes.len()
    }

    /// Whether position `v` holds a node: the mask every read consults.
    pub(crate) fn is_live(&self, v: usize) -> bool {
        self.nodes.get(v).is_some_and(Option::is_some)
    }

    /// The edges out of `v` to live nodes as `(target, weight)`, sorted by target then key;
    /// nothing when `v` holds no node.
    pub(crate) fn forward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        self.live_entries(&self.forward, v)
    }

    /// The edges into `v` from live nodes as `(source, weight)`, sorted by source then key;
    /// nothing when `v` holds no node.
    pub(crate) fn backward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        self.live_entries(&self.backward, v)
    }

    /// Row `v` of `arena` with the entries to removed nodes skipped; empty when `v` holds no
    /// node.
    fn live_entries<'a>(
        &'a self,
        arena: &'a Arena<W>,
        v: usize,
    ) -> impl Iterator<Item = (usize, &'a W)> {
        let (targets, weights): (&[usize], &[W]) = if self.is_live(v) {
            arena.row(v)
        } else {
            (&[], &[])
        };
        targets
            .iter()
            .zip(weights)
            .filter(|(other, _)| self.is_live(**other))
            .map(|(other, weight)| (*other, weight))
    }
}
