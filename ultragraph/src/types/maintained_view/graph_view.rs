/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::{EdgeKey, GraphView, MaintainedView, Projection};

/// Every read answers for the passing nodes and edges; the payloads and the root are the
/// graph's, the state is the graph's.
impl<N, W, P> GraphView<N, W> for MaintainedView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn is_frozen(&self) -> bool {
        self.graph.is_frozen()
    }

    fn is_empty(&self) -> bool {
        self.data.node_count() == 0
    }

    fn contains_node(&self, index: usize) -> bool {
        self.data.passes(index)
    }

    fn get_node(&self, index: usize) -> Option<&N> {
        self.graph
            .get_node(index)
            .filter(|_| self.data.passes(index))
    }

    fn number_nodes(&self) -> usize {
        self.data.node_count()
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.data.has_edge(a, b)
    }

    fn contains_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool {
        self.data.has_edge_keyed(a, b, key)
    }

    fn number_edges(&self) -> usize {
        self.data.edge_count()
    }

    fn get_all_nodes(&self) -> Vec<&N> {
        (0..self.data.slots())
            .filter_map(|v| self.get_node(v))
            .collect()
    }

    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>> {
        self.data
            .passes(source)
            .then(|| self.data.forward_of(source).collect())
    }

    fn get_last_index(&self) -> Option<usize> {
        (0..self.data.slots()).rev().find(|&v| self.data.passes(v))
    }

    fn contains_root_node(&self) -> bool {
        self.get_root_index().is_some()
    }

    fn get_root_node(&self) -> Option<&N> {
        self.get_root_index().and_then(|index| self.get_node(index))
    }

    /// The graph's root, when it passes.
    fn get_root_index(&self) -> Option<usize> {
        self.graph
            .get_root_index()
            .filter(|&index| self.data.passes(index))
    }
}
