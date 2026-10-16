/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Projection;
use crate::{EdgeKey, GraphError, GraphState, UltraGraphContainer};
use alloc::vec::Vec;

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Removes every tombstone and renumbers the live nodes in their order, returning for each
    /// old position its new position, or `None` for a tombstone. See
    /// [`DynamicGraph::compact_ids`](crate::DynamicGraph::compact_ids).
    ///
    /// Every maintained view is rebuilt from its projection and mode at the new positions,
    /// under its identifier.
    ///
    /// # Errors
    /// Returns `GraphError::GraphIsFrozen` in the frozen state and `GraphError::GraphIsLive` in
    /// the live state; the graph is unchanged.
    pub fn compact_ids(&mut self) -> Result<Vec<Option<usize>>, GraphError> {
        let table = match &mut self.state {
            GraphState::Dynamic(g) => Ok(g.compact_ids()),
            GraphState::Live(_) => Err(GraphError::GraphIsLive),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        // Every view is rebuilt from its projection at the new positions.
        let (state, views) = (&self.state, &mut self.views);
        for view in views {
            view.rebuild(state);
        }
        Ok(table)
    }
}
