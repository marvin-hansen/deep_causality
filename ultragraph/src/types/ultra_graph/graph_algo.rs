/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GraphAlgorithms, GraphError, GraphState, UltraGraphContainer};

impl<N, W> GraphAlgorithms<N, W> for UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    /// Finds a cycle in the graph.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.find_cycle(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Checks if the graph contains a cycle.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn has_cycle(&self) -> Result<bool, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.has_cycle(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Performs a topological sort of the graph's nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error from the underlying implementation if the graph contains a cycle.
    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.topological_sort(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Checks if a node `stop_index` is reachable from `start_index`.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.is_reachable(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Calculates the length of the shortest path between two nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.shortest_path_len(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Finds the shortest path (as a sequence of node indices) between two nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.shortest_path(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }
}
