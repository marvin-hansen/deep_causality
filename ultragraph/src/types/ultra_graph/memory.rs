/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Projection;
use crate::{EdgeKey, GraphState, UltraGraphContainer};

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// The bytes the current state's edge storage holds; see the state's own accounting.
    pub(crate) fn edge_storage_bytes(&self) -> usize {
        match &self.state {
            GraphState::Dynamic(g) => g.edge_storage_bytes(),
            GraphState::Live(g) => g.edge_storage_bytes(),
            GraphState::Static(g) => g.edge_storage_bytes(),
        }
    }
}
