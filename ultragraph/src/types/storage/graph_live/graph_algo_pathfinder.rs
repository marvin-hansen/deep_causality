/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::graph_csm_algo_pathfinder::{
    is_reachable_impl, shortest_path_impl, shortest_path_len_impl, shortest_weighted_path_impl,
};
use crate::{EdgeKey, GraphError, LiveGraph, PathfindingGraphAlgorithms};

impl<N, W> PathfindingGraphAlgorithms<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        is_reachable_impl(self, start_index, stop_index)
    }

    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        shortest_path_len_impl(self, start_index, stop_index)
    }

    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        shortest_path_impl(self, start_index, stop_index)
    }

    fn shortest_weighted_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<(Vec<usize>, W)>, GraphError>
    where
        W: Copy + Ord + Default + core::ops::Add<Output = W>,
    {
        shortest_weighted_path_impl(self, start_index, stop_index)
    }
}
