/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::storage::graph_csm::CsrAdjacency;
use crate::{CsmGraph, EdgeKey};
use core::mem::size_of;

/// The bytes one CSR holds: its offsets and targets, `usize` each, and its weights.
fn csr_bytes<W: Clone + Default + EdgeKey>(csr: &CsrAdjacency<W>) -> usize {
    csr.offsets.capacity() * size_of::<usize>()
        + csr.targets.capacity() * size_of::<usize>()
        + csr.weights.capacity() * size_of::<W>()
}

impl<N, W> CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The bytes the edge storage holds: the capacity of the offsets, targets and weights of
    /// the forward and the backward CSR, each times its element size. The node table is not
    /// edge storage and is not counted.
    pub(crate) fn edge_storage_bytes(&self) -> usize {
        csr_bytes(&self.forward_edges) + csr_bytes(&self.backward_edges)
    }
}
