/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    EdgeKey, GraphError, GraphState, ParallelEdges, Projection, ProjectionView, UltraGraphContainer,
};

impl<N, W> UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// A view of the frozen or live graph restricted to what `projection` passes, counting
    /// parallel edges under `mode`. Every algorithm, read and traversal runs on the view;
    /// positions are the graph's. The view borrows the graph.
    ///
    /// # Errors
    /// Returns `GraphError::GraphNotFrozen` in the dynamic state.
    pub fn project<P>(
        &self,
        projection: P,
        mode: ParallelEdges,
    ) -> Result<ProjectionView<'_, N, W, P>, GraphError>
    where
        P: Projection<N, W>,
    {
        match &self.state {
            GraphState::Static(g) => Ok(ProjectionView::frozen(g, projection, mode)),
            GraphState::Live(g) => Ok(ProjectionView::live(g, projection, mode)),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }
}
