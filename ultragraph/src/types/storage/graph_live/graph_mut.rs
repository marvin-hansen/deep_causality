/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_live::Arena;
use crate::{EdgeKey, GraphError, GraphMut, LiveGraph};

impl<N, W> GraphMut<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Adds a node at the next position with an empty row in each arena; the row's first edge
    /// gives it a capacity of one at the arena's end.
    fn add_node(&mut self, node: N) -> Result<usize, GraphError> {
        let index = self.nodes.len();
        self.nodes.push(Some(node));
        self.live_nodes += 1;
        self.forward.push_row();
        self.backward.push_row();
        Ok(index)
    }

    /// # Errors
    /// Returns `GraphError::NodeNotFound` if the position is past the table or removed.
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError> {
        match self.nodes.get_mut(index) {
            Some(slot) if slot.is_some() => {
                *slot = Some(node);
                Ok(())
            }
            _ => Err(GraphError::NodeNotFound(index)),
        }
    }

    /// Removes the node, leaving a tombstone: its rows are emptied, the edge count drops by its
    /// live degree, and its entries in its neighbours' rows stay until those rows are rebuilt,
    /// skipped by every read through the mask. O(degree).
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if the position is past the table or removed.
    fn remove_node(&mut self, index: usize) -> Result<(), GraphError> {
        if !self.is_live(index) {
            return Err(GraphError::NodeNotFound(index));
        }
        // Entries to nodes removed earlier were subtracted then; a self-loop sits in both rows
        // and is one edge, counted with the outgoing ones.
        let outgoing = self.forward_of(index).count();
        let incoming = self
            .backward_of(index)
            .filter(|(source, _)| *source != index)
            .count();
        self.live_edges -= outgoing + incoming;
        self.nodes[index] = None;
        self.live_nodes -= 1;
        self.forward.clear_row(index);
        self.backward.clear_row(index);
        if self.root_index == Some(index) {
            self.root_index = None;
        }
        Ok(())
    }

    /// Adds the edge `(a, b, weight.edge_key())`, or replaces the payload of that edge when it
    /// exists, in `a`'s forward row and `b`'s backward row: a binary search and a shift within
    /// each row, or a rebuild of the row at the arena's end when it is full.
    ///
    /// # Errors
    /// Returns `GraphError::EdgeCreationError` if either end holds no live node.
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError> {
        if !self.is_live(a) || !self.is_live(b) {
            return Err(GraphError::EdgeCreationError {
                source: a,
                target: b,
            });
        }
        let key = weight.edge_key();
        match self.forward.search(a, b, key) {
            Ok(at) => {
                let start = self.forward.rows[a].start;
                self.forward.weights[start + at] = weight.clone();
                if let Ok(back) = self.backward.search(b, a, key) {
                    let start = self.backward.rows[b].start;
                    self.backward.weights[start + back] = weight;
                }
            }
            Err(at) => {
                self.forward.insert_at(a, at, b, weight.clone());
                let back = self
                    .backward
                    .search(b, a, key)
                    .unwrap_or_else(|insert_at| insert_at);
                self.backward.insert_at(b, back, a, weight);
                self.live_edges += 1;
            }
        }
        Ok(())
    }

    /// Removes every edge from `a` to `b`, whatever its key.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if `a` holds no live node.
    /// Returns `GraphError::EdgeNotFoundError` if no edge from `a` to `b` exists.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError> {
        if !self.is_live(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        let (lo, hi) = if self.is_live(b) {
            self.forward.run(a, b)
        } else {
            (0, 0)
        };
        if lo == hi {
            return Err(GraphError::EdgeNotFoundError {
                source: a,
                target: b,
            });
        }
        self.forward.remove_run(a, lo, hi);
        let (back_lo, back_hi) = self.backward.run(b, a);
        self.backward.remove_run(b, back_lo, back_hi);
        self.live_edges -= hi - lo;
        Ok(())
    }

    /// Removes the edge from `a` to `b` with key `key` alone.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if `a` holds no live node.
    /// Returns `GraphError::EdgeNotFoundError` if no edge from `a` to `b` carries `key`.
    fn remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) -> Result<(), GraphError> {
        if !self.is_live(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        let found = if self.is_live(b) {
            self.forward.search(a, b, key).ok()
        } else {
            None
        };
        let Some(at) = found else {
            return Err(GraphError::EdgeNotFoundError {
                source: a,
                target: b,
            });
        };
        self.forward.remove_run(a, at, at + 1);
        if let Ok(back) = self.backward.search(b, a, key) {
            self.backward.remove_run(b, back, back + 1);
        }
        self.live_edges -= 1;
        Ok(())
    }

    /// # Errors
    /// Returns `GraphError::RootNodeAlreadyExists` when a root is already designated.
    fn add_root_node(&mut self, node: N) -> Result<usize, GraphError> {
        if self.root_index.is_some() {
            return Err(GraphError::RootNodeAlreadyExists);
        }
        let index = self.add_node(node)?;
        self.root_index = Some(index);
        Ok(index)
    }

    /// Empties the graph; the next position issued is 0.
    fn clear(&mut self) -> Result<(), GraphError> {
        self.nodes = Vec::new();
        self.live_nodes = 0;
        self.forward = Arena::empty();
        self.backward = Arena::empty();
        self.live_edges = 0;
        self.root_index = None;
        Ok(())
    }
}
