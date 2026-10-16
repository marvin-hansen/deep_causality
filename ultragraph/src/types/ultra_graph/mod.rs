/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

use crate::types::maintained_view::data::ViewData;
use crate::{DynamicGraph, EdgeKey, GraphState, NoProjection, ParallelEdges, Projection};
use alloc::vec::Vec;

mod default;
mod graph_algo;
mod graph_compact;
mod graph_evolve;
mod graph_maintain;
mod graph_mut;
mod graph_project;
mod graph_traversal;
mod graph_view;
mod memory;
mod traversal_iter;

pub type UltraGraph<T> = UltraGraphContainer<T, ()>;
pub type UltraGraphWeighted<T, W> = UltraGraphContainer<T, W>;

/// The graph: its state, the views it maintains, and the parallel-edge mode its direct
/// algorithm calls count under, `Merge` for a graph built by `new` or `with_capacity` and the
/// view's mode for a copy `materialize` returns. The third type parameter is the projection
/// type of its maintained views; the default maintains none.
#[derive(Clone, Debug)]
pub struct UltraGraphContainer<N, W, P = NoProjection>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    state: GraphState<N, W>,
    views: Vec<ViewData<W, P>>,
    mode: ParallelEdges,
}

// Constructors

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    pub fn new() -> Self {
        Self {
            state: GraphState::Dynamic(DynamicGraph::new()),
            views: Vec::new(),
            mode: ParallelEdges::Merge,
        }
    }

    pub fn with_capacity(nodes: usize, edges: Option<usize>) -> Self {
        Self {
            state: GraphState::Dynamic(DynamicGraph::with_capacity(nodes, edges)),
            views: Vec::new(),
            mode: ParallelEdges::Merge,
        }
    }

    /// A graph in the given state with no view, counting parallel edges under `mode`.
    pub(crate) fn with_state(state: GraphState<N, W>, mode: ParallelEdges) -> Self {
        Self {
            state,
            views: Vec::new(),
            mode,
        }
    }

    /// The parallel-edge mode the graph's direct algorithm calls count under.
    pub fn mode(&self) -> ParallelEdges {
        self.mode
    }
}
