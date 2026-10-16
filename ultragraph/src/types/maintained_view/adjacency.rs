/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{EdgeKey, MaintainedView, ParallelEdges, Projection};

/// The view's adjacency: its passing nodes and edges under its mode.
impl<N, W, P> Adjacency<W> for MaintainedView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn slots(&self) -> usize {
        self.data.slots()
    }

    fn passes(&self, v: usize) -> bool {
        self.data.passes(v)
    }

    fn mode(&self) -> ParallelEdges {
        self.data.mode()
    }

    fn out_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.data.forward_of(v)
    }

    fn in_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.data.backward_of(v)
    }
}
