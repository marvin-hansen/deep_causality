/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

/// The default projection type passes every node and every edge, so a projection view under
/// it reads as the graph.
#[test]
fn test_no_projection_passes_everything() {
    assert!(Projection::<u8, u64>::node(&NoProjection, &0));
    assert!(Projection::<u8, u64>::edge(&NoProjection, &0));
    assert!(Projection::<&str, ()>::node(&NoProjection, &"x"));
    let mut g: UltraGraphWeighted<u8, u64> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, 5).unwrap();
    g.freeze();
    let view = g.project(NoProjection, ParallelEdges::Merge).unwrap();
    assert_eq!(view.number_nodes(), 2);
    assert_eq!(view.get_edges(0), Some(vec![(1, &5)]));
}

/// The aliases keep the default: a graph named without a projection type is the container
/// with `NoProjection`.
#[test]
fn test_the_aliases_keep_the_default_projection_type() {
    let g: UltraGraph<u8> = UltraGraph::new();
    let same: UltraGraphContainer<u8, (), NoProjection> = g;
    assert!(same.is_empty());
    let g: UltraGraphWeighted<u8, u64> = UltraGraphWeighted::with_capacity(2, None);
    let same: UltraGraphContainer<u8, u64> = g;
    assert_eq!(same.mode(), ParallelEdges::Merge);
}
