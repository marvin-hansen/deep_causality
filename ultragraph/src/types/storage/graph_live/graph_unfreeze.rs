/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::{DynamicGraph, EdgeKey, LiveGraph, Unfreezable};

impl<N, W> Unfreezable<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Splits the forward rows into per-node lists, stale entries left behind; the backward
    /// lists and the edge count are derived from them. O(V + E).
    fn unfreeze(self) -> DynamicGraph<N, W> {
        let edges: Vec<Vec<(usize, W)>> = (0..self.node_slots())
            .map(|v| {
                self.forward_of(v)
                    .map(|(target, weight)| (target, weight.clone()))
                    .collect()
            })
            .collect();
        DynamicGraph::construct(self.nodes, edges, self.root_index)
    }
}
