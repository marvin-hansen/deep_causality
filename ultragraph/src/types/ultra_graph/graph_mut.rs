/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Projection;
use crate::{DynamicGraph, EdgeKey, GraphError, GraphMut, GraphState, UltraGraphContainer};

impl<N, W, P> GraphMut<N, W> for UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Adds a new node with the given payload to the graph.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` (frozen) state.
    fn add_node(&mut self, node: N) -> Result<usize, GraphError> {
        let index = match &mut self.state {
            GraphState::Dynamic(g) => g.add_node(node),
            GraphState::Live(g) => g.add_node(node),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        self.views_add_node(index);
        Ok(index)
    }

    /// Updates the payload of an existing node at a given index.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` state.
    /// - Returns an error if the node at the given index does not exist.
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError> {
        match &mut self.state {
            GraphState::Dynamic(g) => g.update_node(index, node),
            GraphState::Live(g) => g.update_node(index, node),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        let (state, views) = (&self.state, &mut self.views);
        for view in views {
            view.on_update_node(index, state);
        }
        Ok(())
    }

    /// Removes a node at the given index, also removing all edges connected to it.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` state.
    /// - Returns an error if the node at the given index does not exist.
    fn remove_node(&mut self, index: usize) -> Result<(), GraphError> {
        match &mut self.state {
            GraphState::Dynamic(g) => g.remove_node(index),
            GraphState::Live(g) => g.remove_node(index),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        for view in &mut self.views {
            view.on_remove_node(index);
        }
        Ok(())
    }

    /// Adds a weighted edge between two nodes.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` state.
    /// - Returns an error if either node index is invalid or the edge already exists.
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError> {
        if self.views.is_empty() {
            // The path every graph without a view takes: no clone of the payload.
            return match &mut self.state {
                GraphState::Dynamic(g) => g.add_edge(a, b, weight),
                GraphState::Live(g) => g.add_edge(a, b, weight),
                GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
            };
        }
        let for_views = weight.clone();
        match &mut self.state {
            GraphState::Dynamic(g) => g.add_edge(a, b, weight),
            GraphState::Live(g) => g.add_edge(a, b, weight),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        for view in &mut self.views {
            view.on_add_edge(a, b, &for_views);
        }
        Ok(())
    }

    /// Removes an edge between two nodes.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` state.
    /// - Returns an error if either node index is invalid or the edge does not exist.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError> {
        match &mut self.state {
            GraphState::Dynamic(g) => g.remove_edge(a, b),
            GraphState::Live(g) => g.remove_edge(a, b),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        for view in &mut self.views {
            view.on_remove_edge(a, b);
        }
        Ok(())
    }

    fn remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) -> Result<(), GraphError> {
        match &mut self.state {
            GraphState::Dynamic(g) => g.remove_edge_keyed(a, b, key),
            GraphState::Live(g) => g.remove_edge_keyed(a, b, key),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        for view in &mut self.views {
            view.on_remove_edge_keyed(a, b, key);
        }
        Ok(())
    }

    /// Adds a new node and designates it as the root of the graph.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphIsFrozen` if the graph is in a `Static` (frozen) state.
    fn add_root_node(&mut self, node: N) -> Result<usize, GraphError> {
        let index = match &mut self.state {
            GraphState::Dynamic(g) => g.add_root_node(node),
            GraphState::Live(g) => g.add_root_node(node),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }?;
        self.views_add_node(index);
        Ok(index)
    }

    /// Clears all nodes and edges from the graph, resetting it to an empty, dynamic state from
    /// any state; the next position issued is 0.
    ///
    /// This operation is always successful and will unfreeze the graph if it was frozen or live.
    fn clear(&mut self) -> Result<(), GraphError> {
        // Clearing the graph resets it to a new, empty, dynamic state, regardless of its
        // previous state; every maintained view stays registered and holds nothing.
        self.state = GraphState::Dynamic(DynamicGraph::new());
        for view in &mut self.views {
            view.on_clear();
        }
        Ok(())
    }
}

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Tells every view about the node just added at `index`; nothing to do, and no read of the
    /// node, when the graph maintains no view.
    fn views_add_node(&mut self, index: usize) {
        if self.views.is_empty() {
            return;
        }
        let (state, views) = (&self.state, &mut self.views);
        if let Some(payload) = state.get_node(index) {
            for view in views {
                view.on_add_node(payload);
            }
        }
    }
}
