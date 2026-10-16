/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Projection;
use crate::{EdgeKey, Freezable, GraphState, LiveGraph, UltraGraphContainer, Unfreezable};

impl<N, W, P> UltraGraphContainer<N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// Ensures the graph is in the immutable, performance-optimized `Static` state.
    ///
    /// If the graph is already frozen, this operation is a no-op. From the `Dynamic` state the
    /// conversion is O(V + E) for counting and placement plus a sort of each adjacency list; from
    /// the `Live` state it is O(V + E) with no sort, closing the arenas' holes.
    pub fn freeze(&mut self) {
        // Take ownership of the state, leaving a default in its place.
        let current_state = core::mem::take(&mut self.state);
        self.state = match current_state {
            GraphState::Dynamic(g) => GraphState::Static(g.freeze()),
            GraphState::Live(g) => GraphState::Static(g.freeze()),
            frozen @ GraphState::Static(_) => frozen,
        };
    }

    /// Ensures the graph is in the mutable, `Dynamic` state.
    ///
    /// If the graph is already dynamic, this operation is a no-op. Otherwise it converts the
    /// graph in place, O(V + E); node and edge data must be `Clone`.
    pub fn unfreeze(&mut self) {
        let current_state = core::mem::take(&mut self.state);
        self.state = match current_state {
            GraphState::Static(g) => GraphState::Dynamic(g.unfreeze()),
            GraphState::Live(g) => GraphState::Dynamic(g.unfreeze()),
            dynamic @ GraphState::Dynamic(_) => dynamic,
        };
    }

    /// Ensures the graph is in the `Live` state, mutable row by row and analysable in place.
    ///
    /// If the graph is already live, this operation is a no-op. From the `Static` state the
    /// CSR arrays become the arenas, O(V); from the `Dynamic` state the graph is frozen first,
    /// the work of `freeze`.
    pub fn make_live(&mut self) {
        let current_state = core::mem::take(&mut self.state);
        self.state = match current_state {
            GraphState::Dynamic(g) => GraphState::Live(LiveGraph::from_frozen(g.freeze())),
            GraphState::Static(g) => GraphState::Live(LiveGraph::from_frozen(g)),
            live @ GraphState::Live(_) => live,
        };
    }

    /// True in the `Live` state only.
    pub fn is_live(&self) -> bool {
        matches!(self.state, GraphState::Live(_))
    }

    /// A frozen copy of the graph: the same positions, payloads, tombstones, edges, root and
    /// mode in the `Static` state, with no maintained view. The source keeps its state and its
    /// views and is not changed. The cost is a clone of the current state plus, from the
    /// `Dynamic` or the `Live` state, its `freeze`.
    pub fn freeze_clone(&self) -> UltraGraphContainer<N, W> {
        let state = match &self.state {
            GraphState::Dynamic(g) => GraphState::Static(g.clone().freeze()),
            GraphState::Live(g) => GraphState::Static(g.clone().freeze()),
            GraphState::Static(g) => GraphState::Static(g.clone()),
        };
        UltraGraphContainer::with_state(state, self.mode)
    }
}
