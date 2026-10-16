/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod adjacency;
pub(crate) mod data;
mod graph_algo;
mod graph_traversal;
mod graph_view;

use crate::types::maintained_view::data::ViewData;
use crate::{EdgeKey, ParallelEdges, Projection, UltraGraphContainer};

/// A read handle on a view the graph keeps current: the passing nodes and edges under the
/// projection and mode the view was registered with, borrowed together with the graph, which
/// supplies the payloads and the root. Every read, traversal and algorithm runs on it in every
/// state of the graph.
#[derive(Clone, Copy, Debug)]
pub struct MaintainedView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    graph: &'a UltraGraphContainer<N, W, P>,
    data: &'a ViewData<W, P>,
}

impl<'a, N, W, P> MaintainedView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    pub(crate) fn new(graph: &'a UltraGraphContainer<N, W, P>, data: &'a ViewData<W, P>) -> Self {
        Self { graph, data }
    }

    /// The parallel-edge mode the view's algorithms count under.
    pub fn mode(&self) -> ParallelEdges {
        self.data.mode()
    }
}
