/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod adjacency;
mod backing;
mod graph_algo;
mod graph_traversal;
mod graph_view;

use alloc::vec::Vec;

use crate::types::projection_view::backing::Backing;
use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{
    CsmGraph, DynamicGraph, EdgeKey, Freezable, GraphState, GraphView, LiveGraph, ParallelEdges,
    Projection, UltraGraphContainer,
};

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

    /// An owned graph in the `Static` state holding the nodes and edges the view passes, at the
    /// source's positions, every failing node a tombstone, counting parallel edges under the
    /// view's mode; independent of the source from then on.
    pub fn materialize(&self) -> UltraGraphContainer<N, W> {
        let slots = self.backing.slots();
        let nodes: Vec<Option<N>> = (0..slots).map(|v| self.get_node(v).cloned()).collect();
        let edges: Vec<Vec<(usize, W)>> = (0..slots)
            .map(|v| {
                self.out_edges(v)
                    .map(|(target, weight)| (target, weight.clone()))
                    .collect()
            })
            .collect();
        let frozen = DynamicGraph::from_parts((nodes, edges, self.get_root_index())).freeze();
        UltraGraphContainer::with_state(GraphState::Static(frozen), self.mode)
    }
}
