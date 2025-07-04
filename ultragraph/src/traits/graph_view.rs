/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub trait GraphView<N, W> {
    // State Inspection
    fn is_frozen(&self) -> bool;

    fn is_empty(&self) -> bool;

    // Node Inspection
    fn contains_node(&self, index: usize) -> bool;
    fn get_node(&self, index: usize) -> Option<&N>;
    fn number_nodes(&self) -> usize;

    // Edge Inspection
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    fn number_edges(&self) -> usize;

    /// Returns a vector of immutable references to all active nodes in the graph.
    ///
    /// The order of the nodes in the returned vector is not guaranteed.
    /// This operation is O(V) as it iterates through all possible node slots
    fn get_all_nodes(&self) -> Vec<&N>;

    /// Retrieves a list of all outgoing edges from a given source node.
    /// Returns `None` if the source node does not exist.
    /// The returned vector contains tuples of `(target_node_index, edge_weight_reference)`.
    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>>;

    fn get_last_index(&self) -> Option<usize>;

    // Root Node Inspection
    fn contains_root_node(&self) -> bool;
    fn get_root_node(&self) -> Option<&N>;
    fn get_root_index(&self) -> Option<usize>;
}
