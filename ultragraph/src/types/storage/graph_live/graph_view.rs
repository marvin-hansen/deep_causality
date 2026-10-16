/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::{EdgeKey, GraphView, LiveGraph};

impl<N, W> GraphView<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The Live state is not frozen: it accepts mutations.
    fn is_frozen(&self) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        self.live_nodes == 0
    }

    fn contains_node(&self, index: usize) -> bool {
        self.is_live(index)
    }

    fn get_node(&self, index: usize) -> Option<&N> {
        self.nodes.get(index).and_then(Option::as_ref)
    }

    fn number_nodes(&self) -> usize {
        self.live_nodes
    }

    /// True when `a` and `b` are live and `a`'s row holds an edge to `b`: a binary search by
    /// target, since a live target's entries are never stale.
    fn contains_edge(&self, a: usize, b: usize) -> bool {
        if !self.is_live(a) || !self.is_live(b) {
            return false;
        }
        let (lo, hi) = self.forward.run(a, b);
        lo < hi
    }

    fn contains_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool {
        self.is_live(a) && self.is_live(b) && self.forward.search(a, b, key).is_ok()
    }

    /// The number of edges between live nodes, kept as a count, O(1).
    fn number_edges(&self) -> usize {
        self.live_edges
    }

    fn get_all_nodes(&self) -> Vec<&N> {
        self.nodes.iter().filter_map(Option::as_ref).collect()
    }

    /// The edges out of `source` to live nodes, sorted by target then key; `None` when `source`
    /// holds no node.
    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>> {
        self.is_live(source)
            .then(|| self.forward_of(source).collect())
    }

    fn get_last_index(&self) -> Option<usize> {
        self.nodes.iter().rposition(Option::is_some)
    }

    fn contains_root_node(&self) -> bool {
        self.get_root_index().is_some()
    }

    fn get_root_node(&self) -> Option<&N> {
        self.root_index.and_then(|index| self.get_node(index))
    }

    fn get_root_index(&self) -> Option<usize> {
        self.root_index.filter(|&index| self.is_live(index))
    }
}
