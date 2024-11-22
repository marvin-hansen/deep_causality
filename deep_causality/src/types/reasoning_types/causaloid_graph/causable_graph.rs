// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

/// Implements the CausableGraphExplaining trait for CausaloidGraph.
///
/// This provides default implementations for explaining causaloid graph
/// operations.
///
/// # Type Parameters
///
/// - `T` - Node type that implements Causable + PartialEq
/// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphExplaining<T> for CausaloidGraph<T> where T: Causable + PartialEq {}

/// Implements the CausableGraphReasoning trait for CausaloidGraph.
///
/// This provides default implementations for reasoning about a
/// causaloid graph.
///
/// # Type Parameters
///
/// - `T` - Node type that implements Causable + PartialEq
///
/// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphReasoning<T> for CausaloidGraph<T> where T: Causable + PartialEq {}

/// Implements the CausableGraph trait for CausaloidGraph.
///
/// This provides the core graph manipulation methods for a causaloid graph.
///
/// # Type Parameters
///
/// - `T` - Node type that implements Causable + PartialEq
///
impl<T> CausableGraph<T> for CausaloidGraph<T>
where
    T: Causable + PartialEq,
{
    /// Gets a reference to the underlying CausalGraph.
    ///
    /// # Returns
    ///
    /// Immutable reference to the CausalGraph field.
    ///
    fn get_graph(&self) -> &CausalGraph<T> {
        &self.graph
    }

    /// Adds a new root causaloid node to the graph.
    ///
    /// # Arguments
    ///
    /// * `value` - The node value to add as the root
    ///
    /// # Returns
    ///
    /// The index of the new root node
    ///
    fn add_root_causaloid(&mut self, value: T) -> usize {
        self.graph.add_root_node(value)
    }

    /// Checks if the graph contains a root causaloid node.
    ///
    /// # Returns
    ///
    /// true if the graph has a root node, false otherwise.
    ///
    fn contains_root_causaloid(&self) -> bool {
        self.graph.contains_root_node()
    }

    /// Gets the root causaloid node, if it exists.
    ///
    /// # Returns
    ///
    /// Optional reference to the root causaloid node.
    ///
    fn get_root_causaloid(&self) -> Option<&T> {
        self.graph.get_root_node()
    }

    /// Gets the index of the root causaloid node, if it exists.
    ///
    /// # Returns
    ///
    /// Optional index of the root causaloid node.
    ///
    fn get_root_index(&self) -> Option<usize> {
        self.graph.get_root_index()
    }

    /// Gets the index of the last causaloid node in the graph.
    ///
    /// # Returns
    ///
    /// Ok(index) - The index of the last node if the graph is not empty.
    /// Err - If the graph is empty.
    ///
    fn get_last_index(&self) -> Result<usize, CausalityGraphError> {
        if !self.is_empty() {
            let last_index = self
                .graph
                .get_last_index()
                .expect("Could not get last index");

            Ok(last_index)
        } else {
            Err(CausalityGraphError("Graph is empty".to_string()))
        }
    }

    /// Adds a new causaloid node to the graph.
    ///
    /// # Arguments
    ///
    /// * `value` - The node value to add
    ///
    /// # Returns
    ///
    /// The index of the new node
    ///
    fn add_causaloid(&mut self, value: T) -> usize {
        self.graph.add_node(value)
    }

    /// Checks if the graph contains a causaloid node at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The node index to check
    ///
    /// # Returns
    ///
    /// true if the node exists, false otherwise
    ///
    fn contains_causaloid(&self, index: usize) -> bool {
        self.graph.contains_node(index)
    }

    /// Gets a reference to the causaloid node at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the node to get
    ///
    /// # Returns
    ///
    /// Optional reference to the causaloid node if it exists.
    ///
    fn get_causaloid(&self, index: usize) -> Option<&T> {
        self.graph.get_node(index)
    }

    /// Removes the causaloid node at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the node to remove
    ///
    /// # Returns
    ///
    /// Ok(()) if the node was removed successfully, Err otherwise.
    ///
    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_node(index) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    /// Adds an edge between two causaloid nodes.
    ///
    /// # Arguments
    ///
    /// * `a` - Index of the first node
    /// * `b` - Index of the second node
    ///
    /// # Returns
    ///
    /// Ok(()) if the edge was added successfully, Err otherwise.
    ///
    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    /// Adds a weighted edge between two causaloid nodes.
    ///
    /// # Arguments
    ///
    /// * `a` - Index of the first node
    /// * `b` - Index of the second node
    /// * `weight` - The edge weight
    ///
    /// # Returns
    ///
    /// Ok(()) if the edge was added successfully, Err otherwise.
    ///
    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge_with_weight(a, b, weight) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    /// Checks if an edge exists between two causaloid nodes.
    ///
    /// # Arguments
    ///
    /// * `a` - Index of first node
    /// * `b` - Index of second node
    ///
    /// # Returns
    ///
    /// true if edge exists, false otherwise.
    ///
    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.graph.contains_edge(a, b)
    }

    /// Removes an edge between two causaloid nodes.
    ///
    /// # Arguments
    ///
    /// * `a` - Index of first node
    /// * `b` - Index of second node
    ///
    /// # Returns
    ///
    /// Ok(()) if the edge was removed successfully, Err otherwise.
    ///
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    /// Checks if all causaloid nodes in the graph are active.
    ///
    /// Iterates through all nodes and returns false if any node is not active.
    ///
    /// # Returns
    ///
    /// true if all nodes are active, false otherwise.
    ///
    fn all_active(&self) -> bool {
        for cause in self.graph.get_all_nodes() {
            if !cause.is_active() {
                return false;
            }
        }

        true
    }

    /// Gets the number of active causaloid nodes in the graph.
    ///
    /// Iterates through all nodes and counts the number of active nodes.
    ///
    /// # Returns
    ///
    /// The number of active nodes as a NumericalValue.
    ///
    fn number_active(&self) -> NumericalValue {
        self.graph
            .get_all_nodes()
            .iter()
            .filter(|c| c.is_active())
            .count() as NumericalValue
    }

    /// Gets the percentage of active causaloid nodes in the graph.
    ///
    /// Calculates the percentage by dividing the number of active nodes
    /// by the total number of nodes and multiplying by 100.
    ///
    /// # Returns
    ///
    /// The percentage of active nodes as a NumericalValue.
    ///
    fn percent_active(&self) -> NumericalValue {
        (self.number_active() / self.size() as NumericalValue) * (100 as NumericalValue)
    }

    /// Gets the total number of nodes in the graph.
    ///
    /// # Returns
    ///
    /// The number of nodes.
    ///
    fn size(&self) -> usize {
        self.graph.size()
    }

    /// Checks if the graph is empty (contains no nodes).
    ///
    /// # Returns
    ///
    /// true if the graph is empty, false otherwise.
    ///
    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    /// Clears the graph, removing all nodes and edges.
    ///
    fn clear(&mut self) {
        self.graph.clear();
    }

    /// Gets the total number of edges in the graph.
    ///
    /// # Returns
    ///
    /// The number of edges.
    ///
    fn number_edges(&self) -> usize {
        self.graph.number_edges()
    }

    /// Gets the total number of nodes in the graph.
    ///
    /// # Returns
    ///
    /// The number of nodes.
    ///
    fn number_nodes(&self) -> usize {
        self.graph.number_nodes()
    }
}
