/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EdgeKey, GraphError, GraphTraversal, LiveGraph};

impl<N, W> GraphTraversal<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The live targets of the node's outgoing edges, one per edge, sorted by target then key.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node: past the table or removed.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.is_live(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.forward_of(a).map(|(target, _)| target))
    }

    /// The live sources of the node's incoming edges, one per edge, sorted by source then key.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node: past the table or removed.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.is_live(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.backward_of(a).map(|(source, _)| source))
    }
}
