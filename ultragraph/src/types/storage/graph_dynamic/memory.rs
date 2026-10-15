/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DynamicGraph, EdgeKey};
use alloc::vec::Vec;
use core::mem::size_of;

impl<N, W> DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// The bytes the edge storage holds: the capacity of the outer forward and backward
    /// vectors and of every per-node list, each times its element size. The node table is not
    /// edge storage and is not counted.
    pub(crate) fn edge_storage_bytes(&self) -> usize {
        let forward = self.edges.capacity() * size_of::<Vec<(usize, W)>>()
            + self
                .edges
                .iter()
                .map(|list| list.capacity() * size_of::<(usize, W)>())
                .sum::<usize>();
        let backward = self.backward.capacity() * size_of::<Vec<(usize, W::Key)>>()
            + self
                .backward
                .iter()
                .map(|list| list.capacity() * size_of::<(usize, W::Key)>())
                .sum::<usize>();
        forward + backward
    }
}
