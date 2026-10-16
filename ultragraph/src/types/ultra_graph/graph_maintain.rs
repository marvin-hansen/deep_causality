/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::maintained_view::data::ViewData;
use crate::{EdgeKey, MaintainedView, ParallelEdges, Projection, UltraGraphContainer, ViewId};

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Registers a view of the nodes and edges `projection` passes, counting parallel edges
    /// under `mode`, and returns its identifier. The graph keeps the view current on every
    /// mutation in the Dynamic and the Live state, re-testing only what the mutation touches, so
    /// the projection's tests read the payloads and the parameters fixed here. Every mutation
    /// then pays for each view it has; `freeze` keeps the views, `freeze_clone` leaves them
    /// behind, `compact_ids` rebuilds them at the new positions.
    pub fn maintain(&mut self, projection: P, mode: ParallelEdges) -> ViewId {
        let id = ViewId(self.views.len());
        self.views
            .push(ViewData::new(projection, mode, &self.state));
        id
    }

    /// The view `id` names, in every state, or `None` for an identifier this graph did not issue.
    pub fn view(&self, id: ViewId) -> Option<MaintainedView<'_, N, W, P>> {
        self.views
            .get(id.0)
            .map(|data| MaintainedView::new(self, data))
    }
}
