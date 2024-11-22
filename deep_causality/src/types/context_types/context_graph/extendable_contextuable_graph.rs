// SPDX-License-Identifier: MIT
// Copyright (c) "2023" The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<'l, D, S, T, ST, V> ExtendableContextuableGraph<'l, D, S, T, ST, V>
    for Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Adds a new extra context graph.
    ///
    /// # Parameters
    ///
    /// - `capacity` - Initial capacity for the new context graph
    /// - `default` - Whether to set the new context as default
    ///
    /// # Returns
    ///
    /// The ID of the newly added context graph
    ///
    /// # Behavior
    ///
    /// - Initializes `extra_contexts` map if needed
    /// - Increments `number_of_extra_contexts`
    /// - Inserts new context graph into `extra_contexts` map
    /// - Sets `extra_context_id` to new context ID if `default` is true
    ///
    fn extra_ctx_add_new(&mut self, capacity: usize, default: bool) -> u64 {
        if self.extra_contexts.is_none() {
            self.extra_contexts = Some(HashMap::new());
        }

        let new_context = ultragraph::new_with_matrix_storage(capacity);

        self.number_of_extra_contexts += 1;

        self.extra_contexts
            .as_mut()
            .expect("Failed get a mutable reference to extra_contexts")
            .insert(self.number_of_extra_contexts, new_context);

        if default {
            self.extra_context_id = self.number_of_extra_contexts;
        }

        self.number_of_extra_contexts
    }

    /// Checks if an extra context with the given ID exists.
    ///
    /// # Parameters
    ///
    /// - `idx` - The ID of the context to check
    ///
    /// # Returns
    ///
    /// True if the context exists, false otherwise
    ///
    fn extra_ctx_check_exists(&self, idx: u64) -> bool {
        idx <= self.number_of_extra_contexts
    }

    /// Gets the current extra context ID.
    ///
    /// # Returns
    ///
    /// The ID of the currently active extra context graph.
    ///
    fn extra_ctx_get_current_id(&self) -> u64 {
        self.extra_context_id
    }

    /// Sets the current extra context ID.
    ///
    /// # Parameters
    ///
    /// - `idx` - The ID of the context to set as current
    ///
    /// # Returns
    ///
    /// Result with error if context does not exist
    ///
    /// # Behavior
    ///
    /// - Checks if context with given ID exists
    /// - Sets `extra_context_id` to `idx` if it exists
    /// - Returns error if context does not exist
    ///
    fn extra_ctx_set_current_id(&mut self, idx: u64) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_check_exists(idx) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        self.extra_context_id = idx;

        Ok(())
    }

    /// Unsets the current extra context ID.
    ///
    /// # Returns
    ///
    /// Result with error if no default context exists.
    ///
    /// # Behavior
    ///
    /// - Sets `extra_context_id` to 0
    /// - Returns error if no default context exists
    ///
    fn extra_ctx_unset_current_id(&mut self) -> Result<(), ContextIndexError> {
        self.extra_context_id = 0;

        Ok(())
    }

    /// Adds a node to the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `value` - The node value to add
    ///
    /// # Returns
    ///
    /// The index of the added node, or an error
    ///
    /// # Behavior
    ///
    /// - Gets mutable reference to current extra context
    /// - Calls `add_node` on current extra context
    /// - Returns index or error from current extra context
    ///
    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST, V>,
    ) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context_mut() {
            Ok(ctx) => Ok(ctx.add_node(value)),
            Err(e) => Err(e),
        };
    }

    /// Checks if the current extra context contains a node with the given index.
    ///
    /// # Parameters
    ///
    /// - `index` - The node index to check
    ///
    /// # Returns
    ///
    /// True if the node exists, false otherwise
    ///
    fn extra_ctx_contains_node(&self, index: usize) -> bool {
        return match self.get_current_extra_context() {
            Ok(ctx) => ctx.contains_node(index),
            Err(_) => false,
        };
    }

    /// Gets an immutable reference to a node in the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `index` - The index of the node to get
    ///
    /// # Returns
    ///
    /// A reference to the node, or an error if it doesn't exist
    ///
    /// # Behavior
    ///
    /// - Gets the current extra context
    /// - Calls `get_node` on the current context
    /// - Returns the node reference or an error
    ///
    fn extra_ctx_get_node(
        &self,
        index: usize,
    ) -> Result<&Contextoid<D, S, T, ST, V>, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => match ctx.get_node(index) {
                Some(node) => Ok(node),
                None => Err(ContextIndexError::new(format!(
                    "node {} does not exist",
                    index
                ))),
            },
            Err(e) => Err(e),
        };
    }

    /// Removes a node from the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `index` - The index of the node to remove
    ///
    /// # Returns
    ///
    /// Result with error if node does not exist
    ///
    /// # Behavior
    ///
    /// - Gets mutable reference to current extra context
    /// - Calls `remove_node` on current context
    /// - Returns result from current context
    ///
    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError> {
        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.remove_node(index) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    /// Adds a weighted edge between two nodes in the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `a` - Index of the source node
    /// - `b` - Index of the target node
    /// - `weight` - The weight for the edge
    ///
    /// # Returns
    ///
    /// Result with error if either node does not exist
    ///
    /// # Behavior
    ///
    /// - Checks that both nodes exist
    /// - Gets mutable reference to current context
    /// - Calls `add_edge_with_weight` on context
    /// - Returns result from context
    ///
    fn extra_ctx_add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_contains_node(a) {
            return Err(ContextIndexError(format!("index a {} not found", a)));
        };

        if !self.extra_ctx_contains_node(b) {
            return Err(ContextIndexError(format!("index b {} not found", b)));
        };

        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.add_edge_with_weight(a, b, weight as u64) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    /// Checks if an edge exists between two nodes in the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `a` - Index of the source node
    /// - `b` - Index of the target node
    ///
    /// # Returns
    ///
    /// True if the edge exists, false otherwise
    ///
    /// # Behavior
    ///
    /// - Checks that nodes `a` and `b` exist
    /// - Gets current extra context
    /// - Calls `contains_edge` on context
    /// - Returns result
    ///
    fn extra_ctx_contains_edge(&self, a: usize, b: usize) -> bool {
        if !self.extra_ctx_contains_node(a) {
            return false;
        };

        if !self.extra_ctx_contains_node(b) {
            return false;
        };

        return match self.get_current_extra_context() {
            Ok(ctx) => ctx.contains_edge(a, b),
            Err(_) => false,
        };
    }

    /// Removes an edge between two nodes in the current extra context graph.
    ///
    /// # Parameters
    ///
    /// - `a` - Index of the source node
    /// - `b` - Index of the target node
    ///
    /// # Returns
    ///
    /// Result with error if edge does not exist
    ///
    /// # Behavior
    ///
    /// - Checks that nodes `a` and `b` exist
    /// - Gets mutable reference to current context
    /// - Calls `remove_edge` on context
    /// - Returns result from context
    ///
    fn extra_ctx_remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_contains_node(a) {
            return Err(ContextIndexError("index a not found".into()));
        };

        if !self.extra_ctx_contains_node(b) {
            return Err(ContextIndexError("index b not found".into()));
        };

        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.remove_edge(a, b) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    /// Gets the number of nodes in the current extra context graph.
    ///
    /// # Returns
    ///
    /// The number of nodes, or an error
    ///
    /// # Behavior
    ///
    /// - Gets the current extra context
    /// - Calls `size` on the context
    /// - Returns the size or an error
    ///
    fn extra_ctx_size(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.size()),
            Err(e) => Err(e),
        };
    }

    /// Checks if the current extra context graph is empty.
    ///
    /// # Returns
    ///
    /// Result with true if empty, false if not, or error
    ///
    /// # Behavior
    ///
    /// - Gets the current extra context
    /// - Calls `is_empty` on the context
    /// - Returns the result or error
    ///
    fn extra_ctx_is_empty(&self) -> Result<bool, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.is_empty()),
            Err(e) => Err(e),
        };
    }

    /// Gets the number of nodes in the current extra context graph.
    ///
    /// # Returns
    ///
    /// The number of nodes, or an error
    ///
    /// # Behavior
    ///
    /// - Gets the current extra context
    /// - Calls `number_nodes` on the context
    /// - Returns the number of nodes or an error
    ///
    fn extra_ctx_node_count(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.number_nodes()),
            Err(e) => Err(e),
        };
    }

    /// Gets the number of edges in the current extra context graph.
    ///
    /// # Returns
    ///
    /// The number of edges, or an error
    ///
    /// # Behavior
    ///
    /// - Gets the current extra context
    /// - Calls `number_edges` on the context
    /// - Returns the number of edges or an error
    ///
    fn extra_ctx_edge_count(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.number_edges()),
            Err(e) => Err(e),
        };
    }
}

impl<'l, D, S, T, ST, V> Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Gets an immutable reference to the current extra context graph.
    ///
    /// # Returns
    ///
    /// A reference to the current extra context, or an error
    ///
    /// # Behavior
    ///
    /// - Checks that the extra context ID is set
    /// - Checks that the context exists
    /// - Gets a reference to the context from `extra_contexts` map
    /// - Returns the reference or an error if not found
    ///
    fn get_current_extra_context(
        &self,
    ) -> Result<&ExtraContext<D, S, T, ST, V>, ContextIndexError> {
        if self.extra_context_id == 0 {
            return Err(ContextIndexError::new("context ID not set".into()));
        }

        if !self.extra_ctx_check_exists(self.extra_context_id) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        let ctx = self
            .extra_contexts
            .as_ref()
            .expect("Failed to get a reference to extra_contexts")
            .get(&self.extra_context_id);

        match ctx {
            None => Err(ContextIndexError::new("context does not exists".into())),
            Some(ctx) => Ok(ctx),
        }
    }

    /// Gets a mutable reference to the current extra context graph.
    ///
    /// # Returns
    ///
    /// A mutable reference to the current extra context, or an error
    ///
    /// # Behavior
    ///
    /// - Checks that the extra context ID is set
    /// - Checks that the context exists
    /// - Gets a mutable reference to the context from `extra_contexts` map
    /// - Returns the reference or an error if not found
    ///
    fn get_current_extra_context_mut(
        &mut self,
    ) -> Result<&mut ExtraContext<D, S, T, ST, V>, ContextIndexError> {
        if self.extra_context_id == 0 {
            return Err(ContextIndexError::new("context ID not set".into()));
        }

        if !self.extra_ctx_check_exists(self.extra_context_id) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        let ctx = self
            .extra_contexts
            .as_mut()
            .expect("Failed to get a reference to extra_contexts")
            .get_mut(&self.extra_context_id);

        match ctx {
            None => Err(ContextIndexError::new("context does not exists".into())),
            Some(ctx) => Ok(ctx),
        }
    }
}
