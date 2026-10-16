/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algorithms on a maintained view: the frozen graph's bodies over the view's passing nodes and
//! edges, under the view's mode, in every state of the graph. Expected values are hand counts.

use ultragraph::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    #[default]
    A,
    B,
}

impl EdgeKey for Kind {
    type Key = Kind;
    fn edge_key(&self) -> Kind {
        *self
    }
}

type InScope = (fn(&bool) -> bool, fn(&Kind) -> bool);

fn in_scope() -> InScope {
    (|scoped| *scoped, |_| true)
}

/// Five nodes, all in scope but 4; a 4-cycle 0 → 1 → 2 → 3 → 0 with a parallel edge 0 → 1 of
/// kind B, a chord 1 → 3, and 4 → 0, 2 → 4 out of scope.
fn graph() -> UltraGraphContainer<bool, Kind, InScope> {
    let mut g = UltraGraphContainer::new();
    for scoped in [true, true, true, true, false] {
        g.add_node(scoped).unwrap();
    }
    g.add_edge(0, 1, Kind::A).unwrap();
    g.add_edge(0, 1, Kind::B).unwrap();
    g.add_edge(1, 2, Kind::A).unwrap();
    g.add_edge(2, 3, Kind::A).unwrap();
    g.add_edge(3, 0, Kind::A).unwrap();
    g.add_edge(1, 3, Kind::A).unwrap();
    g.add_edge(4, 0, Kind::A).unwrap();
    g.add_edge(2, 4, Kind::A).unwrap();
    g
}

#[test]
fn test_structural_algorithms_on_a_view_in_every_state() {
    let mut g = graph();
    let id = g.maintain(in_scope(), ParallelEdges::Merge);
    for state in ["dynamic", "live", "frozen"] {
        match state {
            "live" => g.make_live(),
            "frozen" => g.freeze(),
            _ => {}
        }
        let view = g.view(id).unwrap();
        // The four in-scope nodes form one strongly connected component.
        let mut sccs = view.strongly_connected_components().unwrap();
        sccs.iter_mut().for_each(|c| c.sort_unstable());
        assert_eq!(sccs, vec![vec![0, 1, 2, 3]], "{state}");
        assert_eq!(view.has_cycle(), Ok(true), "{state}");
        assert_eq!(view.topological_sort(), Ok(None), "{state}");
        // Undirected: 0-1 (double), 1-2, 2-3, 3-0, 1-3: a single biconnected block, no bridge,
        // no articulation point.
        assert_eq!(view.bridges(), Ok(vec![]), "{state}");
        assert_eq!(view.articulation_points(), Ok(vec![]), "{state}");
        assert_eq!(view.is_reachable(0, 3), Ok(true), "{state}");
        assert_eq!(
            view.is_reachable(0, 4),
            Ok(false),
            "{state}: 4 is out of scope"
        );
        assert_eq!(view.shortest_path(0, 3), Ok(Some(vec![0, 1, 3])), "{state}");
        // Four nodes on the path 2 → 3 → 0 → 1: the length counts nodes.
        assert_eq!(
            view.shortest_path_len(2, 1),
            Ok(Some(4)),
            "{state}: 2 → 3 → 0 → 1"
        );
    }
}

/// The mode decides how the parallel pair 0 → 1 counts in directed betweenness: under `Merge`
/// one connection, under `Distinct` two paths through the same nodes, which doubles the
/// dependency of every pair routed over that edge.
#[test]
fn test_the_views_mode_governs_betweenness() {
    let mut g = graph();
    let merge = g.maintain(in_scope(), ParallelEdges::Merge);
    let distinct = g.maintain(in_scope(), ParallelEdges::Distinct);
    let scores = |id: ViewId| -> Vec<(usize, f64)> {
        let mut s = g
            .view(id)
            .unwrap()
            .betweenness_centrality(true, false)
            .unwrap();
        s.sort_by_key(|(p, _)| *p);
        s
    };
    let m = scores(merge);
    let d = scores(distinct);
    // Merge, directed shortest paths among 0..4 with the chord 1 → 3:
    // 0→2 via 1, 0→3 via 1 (chord), 1→0 via 3 (1→3→0), 2→0 via 3, 2→1 via 3,0 (2→3→0→1),
    // 3→1 via 0, 3→2 via 0,1. Node 1 lies on 0→2, 0→3, 3→2: 3. Node 3 lies on 1→0, 2→0,
    // 2→1: 3. Node 0 lies on 2→1, 3→1, 3→2: 3. Node 2 lies on none: 0.
    assert_eq!(m, vec![(0, 3.0), (1, 3.0), (2, 0.0), (3, 3.0)]);
    // Distinct: the parallel pair 0 → 1 gives two shortest paths wherever a path uses it; the
    // paths 0→2, 0→3, 3→1, 3→2 and 2→1 each double in count but still all pass the same inner
    // nodes, so the dependencies are unchanged: the scores are the same.
    assert_eq!(d, m);
    assert_eq!(g.view(merge).unwrap().mode(), ParallelEdges::Merge);
    assert_eq!(g.view(distinct).unwrap().mode(), ParallelEdges::Distinct);
}

/// Under `Distinct`, undirected bridges see the parallel pair as two links, so 0-1 is never a
/// bridge even when every other link around it is gone.
#[test]
fn test_distinct_keeps_a_parallel_pair_from_being_a_bridge() {
    let mut g: UltraGraphContainer<bool, Kind, InScope> = UltraGraphContainer::new();
    for _ in 0..3 {
        g.add_node(true).unwrap();
    }
    g.add_edge(0, 1, Kind::A).unwrap();
    g.add_edge(0, 1, Kind::B).unwrap();
    g.add_edge(1, 2, Kind::A).unwrap();
    let merge = g.maintain(in_scope(), ParallelEdges::Merge);
    let distinct = g.maintain(in_scope(), ParallelEdges::Distinct);
    assert_eq!(g.view(merge).unwrap().bridges(), Ok(vec![(0, 1), (1, 2)]));
    assert_eq!(g.view(distinct).unwrap().bridges(), Ok(vec![(1, 2)]));
}
