/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::types::storage::graph_csm::CsrAdjacency;
use crate::types::storage::graph_live::Arena;
use crate::types::storage::graph_live::row_entries::RowEntries;
use crate::{EdgeKey, GraphState, ParallelEdges, Projection};

/// What a graph stores for one maintained view: the projection and mode it was registered with,
/// the mask of passing nodes, the passing edges in the Live layout (a forward and a backward
/// arena of rows sorted by other end then key) and the counts. Every mutation of the graph
/// updates it for the nodes and edges the mutation touches; `rebuild` builds it afresh. An
/// entry whose end stopped passing stays in the other end's row and is skipped through the
/// mask, as in the Live state.
#[derive(Clone, Debug)]
pub(crate) struct ViewData<W, P> {
    projection: P,
    mode: ParallelEdges,
    passes: Vec<bool>,
    forward: Arena<W>,
    backward: Arena<W>,
    nodes: usize,
    edges: usize,
}

impl<W, P> ViewData<W, P>
where
    W: Clone + Default + EdgeKey,
{
    pub(crate) fn mode(&self) -> ParallelEdges {
        self.mode
    }

    pub(crate) fn slots(&self) -> usize {
        self.passes.len()
    }

    pub(crate) fn passes(&self, v: usize) -> bool {
        self.passes.get(v).copied().unwrap_or(false)
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges
    }

    /// The passing edges out of `v` as `(target, weight)`, sorted by target then key; nothing
    /// when `v` does not pass.
    pub(crate) fn forward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        self.passing_entries(&self.forward, v)
    }

    /// The passing edges into `v` as `(source, weight)`, sorted by source then key; nothing
    /// when `v` does not pass.
    pub(crate) fn backward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        self.passing_entries(&self.backward, v)
    }

    /// Whether `a` passes and its forward row holds an edge to `b`, any key.
    pub(crate) fn has_edge(&self, a: usize, b: usize) -> bool {
        if !self.passes(a) || !self.passes(b) {
            return false;
        }
        let (lo, hi) = self.forward.run(a, b);
        lo < hi
    }

    pub(crate) fn has_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool {
        self.passes(a) && self.passes(b) && self.forward.search(a, b, key).is_ok()
    }

    fn passing_entries<'a>(
        &'a self,
        arena: &'a Arena<W>,
        v: usize,
    ) -> impl Iterator<Item = (usize, &'a W)> {
        let (targets, weights): (&[usize], &[W]) = if self.passes(v) {
            arena.row(v)
        } else {
            (&[], &[])
        };
        RowEntries::new(targets, weights, |other| self.passes(other))
    }

    /// Inserts the passing edge `(a, b, weight)` into both rows, or replaces its payload. A row
    /// a node emptied when it left the view is refilled here when the node returns, so each side
    /// is inserted where it is missing, and the edge counts once unless both sides held it.
    fn put(&mut self, a: usize, b: usize, weight: W) {
        let key = weight.edge_key();
        let forward_held = match self.forward.search(a, b, key) {
            Ok(at) => {
                let start = self.forward.rows[a].start;
                self.forward.weights[start + at] = weight.clone();
                true
            }
            Err(at) => {
                self.forward.insert_at(a, at, b, weight.clone());
                false
            }
        };
        let backward_held = match self.backward.search(b, a, key) {
            Ok(back) => {
                let start = self.backward.rows[b].start;
                self.backward.weights[start + back] = weight;
                true
            }
            Err(back) => {
                self.backward.insert_at(b, back, a, weight);
                false
            }
        };
        if !(forward_held && backward_held) {
            self.edges += 1;
        }
    }

    /// Takes node `v` out of the view: its edges leave the count, its rows are emptied, its
    /// entries in other rows stay behind the mask.
    fn drop_node(&mut self, v: usize) {
        if !self.passes(v) {
            return;
        }
        let outgoing = self.forward_of(v).count();
        let incoming = self
            .backward_of(v)
            .filter(|(source, _)| *source != v)
            .count();
        self.edges -= outgoing + incoming;
        self.nodes -= 1;
        self.passes[v] = false;
        self.forward.clear_row(v);
        self.backward.clear_row(v);
    }
}

impl<W, P> ViewData<W, P>
where
    W: Clone + Default + EdgeKey,
{
    /// A view of `graph` under `projection` and `mode`, built from every node and edge.
    pub(crate) fn new<N>(projection: P, mode: ParallelEdges, graph: &GraphState<N, W>) -> Self
    where
        N: Clone,
        P: Projection<N, W>,
    {
        let mut view = Self {
            projection,
            mode,
            passes: Vec::new(),
            forward: Arena::empty(),
            backward: Arena::empty(),
            nodes: 0,
            edges: 0,
        };
        view.rebuild(graph);
        view
    }

    /// Rebuilds the view from every node and edge of `graph`, at the graph's positions: the
    /// passing edges of each passing node, sorted by other end then key, adopted as rows with no
    /// spare capacity.
    pub(crate) fn rebuild<N>(&mut self, graph: &GraphState<N, W>)
    where
        N: Clone,
        P: Projection<N, W>,
    {
        let slots = graph.node_slots();
        self.passes = (0..slots)
            .map(|v| {
                graph
                    .get_node(v)
                    .is_some_and(|node| self.projection.node(node))
            })
            .collect();
        self.nodes = self.passes.iter().filter(|p| **p).count();
        let mut forward: Vec<Vec<(usize, W)>> = (0..slots).map(|_| Vec::new()).collect();
        let mut backward: Vec<Vec<(usize, W)>> = (0..slots).map(|_| Vec::new()).collect();
        for v in (0..slots).filter(|v| self.passes[*v]) {
            for (target, weight) in graph.edges_of(v) {
                if self.passes[target] && self.projection.edge(weight) {
                    forward[v].push((target, weight.clone()));
                    backward[target].push((v, weight.clone()));
                }
            }
        }
        self.edges = forward.iter().map(Vec::len).sum();
        self.forward = Arena::adopt(csr_of(forward));
        self.backward = Arena::adopt(csr_of(backward));
    }

    /// A node added at the next position: it passes when its payload does.
    pub(crate) fn on_add_node<N>(&mut self, payload: &N)
    where
        P: Projection<N, W>,
    {
        let passes = self.projection.node(payload);
        self.passes.push(passes);
        self.nodes += usize::from(passes);
        self.forward.push_row();
        self.backward.push_row();
    }

    /// The payload of `v` replaced: the node is re-tested; one leaving the view takes its edges
    /// with it, one entering it brings its passing edges back from `graph`.
    pub(crate) fn on_update_node<N>(&mut self, v: usize, graph: &GraphState<N, W>)
    where
        N: Clone,
        P: Projection<N, W>,
    {
        let now = graph
            .get_node(v)
            .is_some_and(|node| self.projection.node(node));
        if now == self.passes(v) {
            return;
        }
        if !now {
            self.drop_node(v);
            return;
        }
        self.passes[v] = true;
        self.nodes += 1;
        let outgoing: Vec<(usize, W)> = graph
            .edges_of(v)
            .into_iter()
            .filter(|(target, weight)| self.passes(*target) && self.projection.edge(weight))
            .map(|(target, weight)| (target, weight.clone()))
            .collect();
        for (target, weight) in outgoing {
            self.put(v, target, weight);
        }
        for source in graph.sources_of(v) {
            if source == v || !self.passes(source) {
                continue;
            }
            let incoming: Vec<W> = graph
                .edges_of(source)
                .into_iter()
                .filter(|(target, weight)| *target == v && self.projection.edge(weight))
                .map(|(_, weight)| weight.clone())
                .collect();
            for weight in incoming {
                self.put(source, v, weight);
            }
        }
    }

    /// Node `v` removed: it fails from now on and its edges leave the count.
    pub(crate) fn on_remove_node(&mut self, v: usize) {
        self.drop_node(v);
    }

    /// The edge `(a, b, weight.edge_key())` added or its payload replaced: it is in the view
    /// when both ends pass and the payload passes, and out of it otherwise.
    pub(crate) fn on_add_edge<N>(&mut self, a: usize, b: usize, weight: &W)
    where
        P: Projection<N, W>,
    {
        if self.passes(a) && self.passes(b) && self.projection.edge(weight) {
            self.put(a, b, weight.clone());
        } else {
            self.on_remove_edge_keyed(a, b, weight.edge_key());
        }
    }

    /// Every edge from `a` to `b` removed, from both rows, whether or not the ends pass.
    pub(crate) fn on_remove_edge(&mut self, a: usize, b: usize) {
        let (lo, hi) = self.forward.run(a, b);
        if lo < hi {
            if self.passes(a) && self.passes(b) {
                self.edges -= hi - lo;
            }
            self.forward.remove_run(a, lo, hi);
        }
        let (lo, hi) = self.backward.run(b, a);
        if lo < hi {
            self.backward.remove_run(b, lo, hi);
        }
    }

    /// The edge from `a` to `b` with `key` removed, from both rows.
    pub(crate) fn on_remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) {
        if let Ok(at) = self.forward.search(a, b, key) {
            if self.passes(a) && self.passes(b) {
                self.edges -= 1;
            }
            self.forward.remove_run(a, at, at + 1);
        }
        if let Ok(back) = self.backward.search(b, a, key) {
            self.backward.remove_run(b, back, back + 1);
        }
    }

    /// The graph cleared: the view stays registered and holds nothing.
    pub(crate) fn on_clear(&mut self) {
        self.passes = Vec::new();
        self.forward = Arena::empty();
        self.backward = Arena::empty();
        self.nodes = 0;
        self.edges = 0;
    }
}

/// Per-node lists, each sorted by other end then key, as one CSR.
fn csr_of<W: Clone + Default + EdgeKey>(mut lists: Vec<Vec<(usize, W)>>) -> CsrAdjacency<W> {
    let total: usize = lists.iter().map(Vec::len).sum();
    let mut offsets = Vec::with_capacity(lists.len() + 1);
    let mut targets = Vec::with_capacity(total);
    let mut weights = Vec::with_capacity(total);
    offsets.push(0);
    for list in &mut lists {
        list.sort_by_key(|(other, weight)| (*other, weight.edge_key()));
        for (other, weight) in list.drain(..) {
            targets.push(other);
            weights.push(weight);
        }
        offsets.push(targets.len());
    }
    CsrAdjacency {
        offsets,
        targets,
        weights,
    }
}
