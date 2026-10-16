/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::{
    graph_csm_algo_articulation_points::articulation_points_impl,
    graph_csm_algo_biconnected_components::biconnected_components_impl,
    graph_csm_algo_bridges::bridges_impl,
    graph_csm_algo_structural::strongly_connected_components_impl,
};
use crate::{EdgeKey, GraphError, LiveGraph, StructuralGraphAlgorithms};

impl<N, W> StructuralGraphAlgorithms<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        strongly_connected_components_impl(self)
    }

    fn articulation_points(&self) -> Result<Vec<usize>, GraphError> {
        articulation_points_impl(self)
    }

    fn bridges(&self) -> Result<Vec<(usize, usize)>, GraphError> {
        bridges_impl(self)
    }

    fn biconnected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        biconnected_components_impl(self)
    }
}
