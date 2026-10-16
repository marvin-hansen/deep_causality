/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::graph_csm_algo_topological::{
    find_cycle_impl, has_cycle_impl, topological_sort_impl,
};
use crate::{EdgeKey, GraphError, LiveGraph, TopologicalGraphAlgorithms};

// Every algorithm runs the body the frozen graph runs, over the Live state's adjacency.

impl<N, W> TopologicalGraphAlgorithms<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        find_cycle_impl(self)
    }

    fn has_cycle(&self) -> Result<bool, GraphError> {
        has_cycle_impl(self)
    }

    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        topological_sort_impl(self)
    }
}
