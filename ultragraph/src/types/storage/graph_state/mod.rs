/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

use alloc::vec::Vec;

use crate::{CsmGraph, DynamicGraph, EdgeKey, GraphTraversal, GraphView, LiveGraph};

#[derive(Clone, Debug)]
pub enum GraphState<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    Dynamic(DynamicGraph<N, W>), // The "unfrozen" state, for mutation
    Live(LiveGraph<N, W>),       // Mutable row by row and analysable in place
    Static(CsmGraph<N, W>),      // The "frozen" state, for analysis
}

// The reads a maintained view's upkeep makes of whichever state the graph is in.
impl<N, W> GraphState<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// One past the highest position ever issued, tombstones included.
    pub(crate) fn node_slots(&self) -> usize {
        match self {
            Self::Dynamic(g) => g.table_len(),
            Self::Live(g) => g.node_slots(),
            Self::Static(g) => g.node_slots(),
        }
    }

    pub(crate) fn get_node(&self, v: usize) -> Option<&N> {
        match self {
            Self::Dynamic(g) => g.get_node(v),
            Self::Live(g) => g.get_node(v),
            Self::Static(g) => g.get_node(v),
        }
    }

    /// The edges out of `v` as `(target, weight)`, empty when `v` holds no node.
    pub(crate) fn edges_of(&self, v: usize) -> Vec<(usize, &W)> {
        let edges = match self {
            Self::Dynamic(g) => g.get_edges(v),
            Self::Live(g) => g.get_edges(v),
            Self::Static(g) => g.get_edges(v),
        };
        edges.unwrap_or_default()
    }

    /// The distinct sources of the edges into `v`, empty when `v` holds no node.
    pub(crate) fn sources_of(&self, v: usize) -> Vec<usize> {
        let mut sources: Vec<usize> = match self {
            Self::Dynamic(g) => g
                .inbound_edges(v)
                .map(Iterator::collect)
                .unwrap_or_default(),
            Self::Live(g) => g
                .inbound_edges(v)
                .map(Iterator::collect)
                .unwrap_or_default(),
            Self::Static(g) => g
                .inbound_edges(v)
                .map(Iterator::collect)
                .unwrap_or_default(),
        };
        sources.sort_unstable();
        sources.dedup();
        sources
    }
}
