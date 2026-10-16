/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::graph_csm_algo_centrality::{
    betweenness_centrality_impl, pathway_betweenness_centrality_impl,
};
use crate::{CentralityGraphAlgorithms, EdgeKey, GraphError, LiveGraph};

impl<N, W> CentralityGraphAlgorithms<N, W> for LiveGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn betweenness_centrality(
        &self,
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        betweenness_centrality_impl(self, directed, normalized)
    }

    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        pathway_betweenness_centrality_impl(self, pathways, directed, normalized)
    }
}
