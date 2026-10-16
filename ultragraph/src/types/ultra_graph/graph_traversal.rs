/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Projection;
use crate::types::ultra_graph::traversal_iter::TraversalIter;
use crate::{EdgeKey, GraphError, GraphState, GraphTraversal, UltraGraphContainer};

impl<N, W, P> GraphTraversal<N, W> for UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Returns an iterator over the targets of the edges out of node `a`, one entry per edge, in
    /// every state. The frozen state reads its CSR slice, the dynamic state the node's forward
    /// list, the live state the node's row.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node, in every state.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.outbound_edges(a).map(TraversalIter::Frozen),
            GraphState::Dynamic(g) => g.outbound_edges(a).map(TraversalIter::Dynamic),
            GraphState::Live(g) => g.outbound_edges(a).map(TraversalIter::Live),
        }
    }

    /// Returns an iterator over the sources of the edges into node `a`, one entry per edge, in
    /// every state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node, in every state.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.inbound_edges(a).map(TraversalIter::Frozen),
            GraphState::Dynamic(g) => g.inbound_edges(a).map(TraversalIter::Dynamic),
            GraphState::Live(g) => g.inbound_edges(a).map(TraversalIter::Live),
        }
    }
}
