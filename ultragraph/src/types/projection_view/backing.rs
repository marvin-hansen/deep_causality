/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CsmGraph, EdgeKey, GraphView, LiveGraph};

/// The graph a projection view reads: the frozen CSR or the live arenas. Both answer the same
/// five questions, so the view's adjacency, reads and traversals are written once.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Backing<'a, N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    Frozen(&'a CsmGraph<N, W>),
    Live(&'a LiveGraph<N, W>),
}

/// One iterator type over the two backings' edge iterators.
pub(crate) enum EdgeIter<F, L> {
    Frozen(F),
    Live(L),
}

impl<'a, W, F, L> Iterator for EdgeIter<F, L>
where
    F: Iterator<Item = (usize, &'a W)>,
    L: Iterator<Item = (usize, &'a W)>,
    W: 'a,
{
    type Item = (usize, &'a W);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Frozen(iter) => iter.next(),
            Self::Live(iter) => iter.next(),
        }
    }
}

impl<N, W> Backing<'_, N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The number of positions in the node table, tombstones included.
    pub(crate) fn slots(&self) -> usize {
        match self {
            Self::Frozen(g) => g.node_slots(),
            Self::Live(g) => g.node_slots(),
        }
    }

    pub(crate) fn get_node(&self, v: usize) -> Option<&N> {
        match self {
            Self::Frozen(g) => g.get_node(v),
            Self::Live(g) => g.get_node(v),
        }
    }

    pub(crate) fn root_index(&self) -> Option<usize> {
        match self {
            Self::Frozen(g) => g.get_root_index(),
            Self::Live(g) => g.get_root_index(),
        }
    }

    /// The edges out of `v` to live nodes as `(target, weight)`, sorted by target then key.
    pub(crate) fn forward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        match self {
            Self::Frozen(g) => EdgeIter::Frozen(g.forward_of(v)),
            Self::Live(g) => EdgeIter::Live(g.forward_of(v)),
        }
    }

    /// The edges into `v` from live nodes as `(source, weight)`, sorted by source then key.
    pub(crate) fn backward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        match self {
            Self::Frozen(g) => EdgeIter::Frozen(g.backward_of(v)),
            Self::Live(g) => EdgeIter::Live(g.backward_of(v)),
        }
    }
}
