/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

/// A node test and an edge test as plain function pointers.
type Tests = (fn(&u8) -> bool, fn(&()) -> bool);

/// Identifiers are issued in order, compare by value, and name the view they came from.
#[test]
fn test_view_ids_name_their_views() {
    let mut g: UltraGraphContainer<u8, (), Tests> = UltraGraphContainer::new();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    let odd = g.maintain((|n: &u8| n % 2 == 1, |_: &()| true), ParallelEdges::Merge);
    let even = g.maintain(
        (|n: &u8| n.is_multiple_of(2), |_: &()| true),
        ParallelEdges::Merge,
    );
    assert_ne!(odd, even);
    assert_eq!(odd, odd.clone());
    assert_eq!(format!("{odd:?}"), "ViewId(0)");
    assert_eq!(format!("{even:?}"), "ViewId(1)");
    assert!(g.view(odd).unwrap().contains_node(0));
    assert!(!g.view(odd).unwrap().contains_node(1));
    assert!(g.view(even).unwrap().contains_node(1));
}
