/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::CsrAdjacency;
use crate::types::storage::graph_live::Arena;
use crate::{CsmGraph, EdgeKey, Freezable, LiveGraph};

impl<N, W> Freezable<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Closes the holes and drops the spare capacity: each row's live entries are copied in
    /// order into fresh CSR arrays, stale entries left behind. O(V + E), no sort, since every
    /// row is already sorted by target then key.
    fn freeze(self) -> CsmGraph<N, W> {
        let forward = self.pack(&self.forward);
        let backward = self.pack(&self.backward);
        CsmGraph::construct(
            self.nodes,
            forward,
            backward,
            self.root_index,
            self.live_nodes,
        )
    }
}

impl<N, W> LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// The CSR of `arena`'s live entries: offsets reserved for one past the positions, targets
    /// and weights of exactly the edge count.
    fn pack(&self, arena: &Arena<W>) -> CsrAdjacency<W> {
        let slots = self.node_slots();
        let mut offsets = Vec::with_capacity(slots + 1);
        let mut targets = Vec::with_capacity(self.live_edges);
        let mut weights = Vec::with_capacity(self.live_edges);
        offsets.push(0);
        for v in 0..slots {
            for (other, weight) in self.live_entries(arena, v) {
                targets.push(other);
                weights.push(weight.clone());
            }
            offsets.push(targets.len());
        }
        CsrAdjacency {
            offsets,
            targets,
            weights,
        }
    }
}
