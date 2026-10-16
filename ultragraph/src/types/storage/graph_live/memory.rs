/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::mem::size_of;

use crate::types::storage::graph_live::{Arena, Row};
use crate::{EdgeKey, LiveGraph};

/// The bytes one arena holds: its targets, weights and rows, each at capacity.
fn arena_bytes<W: Clone + Default + EdgeKey>(arena: &Arena<W>) -> usize {
    arena.targets.capacity() * size_of::<usize>()
        + arena.weights.capacity() * size_of::<W>()
        + arena.rows.capacity() * size_of::<Row>()
}

impl<N, W> LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The bytes the edge storage holds: the capacity of the targets, weights and rows of the
    /// forward and the backward arena, each times its element size, holes and spare capacity
    /// included. The node table is not edge storage and is not counted.
    pub(crate) fn edge_storage_bytes(&self) -> usize {
        arena_bytes(&self.forward) + arena_bytes(&self.backward)
    }
}
