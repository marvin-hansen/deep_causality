/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Projection;

/// The projection type of a graph that maintains no view: the default of the container's
/// third type parameter. It passes every node and every edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoProjection;

impl<N, W> Projection<N, W> for NoProjection {
    fn node(&self, _node: &N) -> bool {
        true
    }

    fn edge(&self, _edge: &W) -> bool {
        true
    }
}
