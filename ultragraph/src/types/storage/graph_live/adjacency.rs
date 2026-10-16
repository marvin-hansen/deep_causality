/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{EdgeKey, LiveGraph, ParallelEdges};

/// The simple projection of the Live state: every live node and every edge between live nodes,
/// under `Merge`, read through the rows and the mask.
impl<N, W> Adjacency<W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn slots(&self) -> usize {
        self.node_slots()
    }

    fn passes(&self, v: usize) -> bool {
        self.is_live(v)
    }

    fn mode(&self) -> ParallelEdges {
        ParallelEdges::Merge
    }

    fn out_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.forward_of(v)
    }

    fn in_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.backward_of(v)
    }

    fn passing_count(&self) -> usize {
        self.live_nodes
    }
}
