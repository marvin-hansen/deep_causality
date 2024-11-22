// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausalGraph,
    NumericalValue,
};

mod causable_graph;
mod default;

/// CausaloidGraph struct representing a causal graph of Causable nodes.
///
/// # Type Parameters
///
/// - `T` - Node type that implements Causable + PartialEq
///
/// # Fields
///
/// - `graph` - Underlying CausalGraph storing nodes and edges
///
/// # Trait Implementations
///
/// - `Clone` - Clone implementation
///
#[derive(Clone)]
pub struct CausaloidGraph<T>
where
    T: Causable + PartialEq,
{
    graph: CausalGraph<T>,
}

impl<T> CausaloidGraph<T>
where
    T: Causable + PartialEq,
{
    /// Creates a new CausaloidGraph with default capacity.
    ///
    /// Initializes a new CausaloidGraph with an underlying CausalGraph
    /// with default capacity of 500 nodes.
    ///
    /// # Returns
    ///
    /// New CausaloidGraph instance.
    ///
    pub fn new() -> Self {
        Self {
            graph: ultragraph::new_with_matrix_storage(500),
        }
    }

    /// Creates a new CausaloidGraph with specified capacity.
    ///
    /// Initializes a new CausaloidGraph with an underlying CausalGraph
    /// with the given capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The node capacity of the underlying CausalGraph
    ///
    /// # Returns
    ///
    /// New CausaloidGraph instance with the specified capacity.
    ///
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            graph: ultragraph::new_with_matrix_storage(capacity),
        }
    }
}
