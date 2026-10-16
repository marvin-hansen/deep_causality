/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod adjacency;
mod backing;
mod graph_algo;
mod graph_traversal;
mod graph_view;

use crate::types::projection_view::backing::Backing;
use crate::{CsmGraph, EdgeKey, LiveGraph, ParallelEdges, Projection};

/// A view of a frozen or live graph restricted to the nodes and edges a [`Projection`] passes,
/// with the parallel-edge mode its algorithms count under. Positions are those of the graph; a
/// failing node reads as absent. The view borrows the graph, so the graph cannot change while it
/// lives.
#[derive(Clone, Debug)]
pub struct ProjectionView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    backing: Backing<'a, N, W>,
    projection: P,
    mode: ParallelEdges,
}

impl<'a, N, W, P> ProjectionView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    pub(crate) fn frozen(graph: &'a CsmGraph<N, W>, projection: P, mode: ParallelEdges) -> Self {
        Self {
            backing: Backing::Frozen(graph),
            projection,
            mode,
        }
    }

    pub(crate) fn live(graph: &'a LiveGraph<N, W>, projection: P, mode: ParallelEdges) -> Self {
        Self {
            backing: Backing::Live(graph),
            projection,
            mode,
        }
    }

    /// The parallel-edge mode the view's algorithms count under.
    pub fn mode(&self) -> ParallelEdges {
        self.mode
    }
}
